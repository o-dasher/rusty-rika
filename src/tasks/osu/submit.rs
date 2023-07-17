use std::{collections::HashSet, sync::Arc};

use derive_more::From;
use log::info;
use rosu_pp::{osu::OsuPerformanceAttributes, OsuPP};
use rosu_v2::prelude::GameMode;

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
        SELECT s.map_id FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE s.osu_user_id = ? AND s.mode = ?
        ",
        osu_id,
        mode_bits
    )
    .fetch_all(db)
    .await?;

    let existing_scores: HashSet<_> = rika_osu_scores.into_iter().map(|s| s.map_id).collect();

    let mut tx = db.begin().await?;

    for (i, score) in osu_scores.iter().enumerate() {
        let Some(score_id) = score.score_id else {
            continue;
        };

        if existing_scores.contains(&(score.map_id)) {
            continue;
        }

        let beatmap_file = beatmap_cache.get_beatmap_file(score.map_id).await?;
        let beatmap_rosu = rosu_pp::Beatmap::from_bytes(&beatmap_file).await?;

        let OsuPerformanceAttributes {
            pp_aim,
            pp_speed,
            pp_flashlight,
            pp_acc,
            pp,
            ..
        } = OsuPP::new(&beatmap_rosu)
            .mods(score.mods.into())
            .combo(score.max_combo as usize)
            .n_misses(score.statistics.count_miss as usize)
            .n300(score.statistics.count_300 as usize)
            .n100(score.statistics.count_100 as usize)
            .n50(score.statistics.count_50 as usize)
            .calculate();

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

        info!("Processed score number {} for {osu_id}", i + 1);
    }

    tx.commit().await?;

    Ok(())
}
