use std::{collections::HashSet, sync::Arc};

use derive_more::From;
use itertools::Itertools;
use log::info;
use rosu_pp::{osu::OsuPerformanceAttributes, OsuPP};
use rosu_v2::prelude::{GameMode, Score};

use crate::{commands::CommandReturn, RikaData};

#[derive(From)]
pub enum SubmissionID {
    ByStoredID(u32),
    ByUsername(String),
}

pub async fn submit_scores(data: &Arc<RikaData>, osu_id: impl Into<SubmissionID>) -> CommandReturn {
    let RikaData {
        db,
        rosu,
        beatmap_cache,
        ..
    } = data.as_ref();

    let mode = GameMode::Osu;
    let mode_bits = mode as i16;

    let osu_id = match osu_id.into() {
        SubmissionID::ByStoredID(id) => id,
        SubmissionID::ByUsername(username) => rosu.user(username).await?.user_id,
    };

    let osu_scores = rosu.user_scores(osu_id).limit(100).mode(mode).await?;

    let rika_osu_scores = sqlx::query!(
        "
        SELECT s.id FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE s.osu_user_id = ? AND s.mode = ?
        ",
        osu_id,
        mode_bits
    )
    .fetch_all(db)
    .await?;

    let existing_scores: HashSet<_> = rika_osu_scores.into_iter().map(|s| s.id).collect();
    let new_scores = osu_scores
        .iter()
        .filter_map(|s| {
            s.score_id.and_then(|score_id| {
                let is_new = !existing_scores.contains(&score_id);

                is_new.then(|| (score_id, s))
            })
        })
        .collect_vec();

    if new_scores.is_empty() {
        return Ok(());
    }

    let mut performance_information: Vec<(OsuPerformanceAttributes, (&Score, &u64))> = vec![];

    for (i, (score_id, score)) in new_scores.iter().enumerate() {
        let beatmap_file = beatmap_cache.get_beatmap_file(score.map_id).await?;
        let beatmap_rosu = rosu_pp::Beatmap::from_bytes(&beatmap_file).await?;

        let performance_attributes = OsuPP::new(&beatmap_rosu)
            .mods(score.mods.into())
            .combo(score.max_combo as usize)
            .n_misses(score.statistics.count_miss as usize)
            .n300(score.statistics.count_300 as usize)
            .n100(score.statistics.count_100 as usize)
            .n50(score.statistics.count_50 as usize)
            .calculate();

        performance_information.push((performance_attributes, (*score, score_id)));

        info!("Processed score number {} for {osu_id}", i + 1);
    }

    let mut tx = db.begin().await?;

    // Can't we do all this in a single query at this point? i think so? i am not sure.
    // I mean, anyways this takes at most 30ms the chances of any deadlocks here are minimal.
    for (
        OsuPerformanceAttributes {
            pp,
            pp_acc,
            pp_aim,
            pp_flashlight,
            pp_speed,
            ..
        },
        (score, score_id),
    ) in performance_information
    {
        sqlx::query!(
            "
            INSERT INTO osu_score (id, osu_user_id, map_id, mods, mode)
            VALUES (?, ?, ?, ?, ?)
            ",
            score_id,
            osu_id,
            score.map_id,
            score.mods.bits(),
            mode_bits
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "
            INSERT INTO osu_performance (id, aim, speed, flashlight, accuracy, overall)
            VALUES (?, ?, ?, ?, ?, ?)
            ",
            score_id,
            pp_aim,
            pp_speed,
            pp_flashlight,
            pp_acc,
            pp
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query!(
        "
        DELETE FROM osu_score
        WHERE id NOT IN (
            SELECT top_100.id
            FROM (
                SELECT id
                FROM osu_score
                WHERE osu_user_id = ?
                ORDER BY created_at DESC
                LIMIT 100
            ) as top_100
        ) AND osu_user_id = ?
        ",
        &osu_id,
        &osu_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
