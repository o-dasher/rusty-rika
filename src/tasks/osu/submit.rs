use std::{collections::HashSet, sync::Arc};

use derive_more::From;
use itertools::Itertools;
use log::info;
use paste::paste;
use rosu_pp::{
    mania::ManiaPerformanceAttributes, osu::OsuPerformanceAttributes,
    taiko::TaikoPerformanceAttributes, ManiaPP, OsuPP, TaikoPP,
};
use rosu_v2::prelude::{GameMode, Score};

use crate::{commands::CommandReturn, RikaData};

#[derive(From)]
pub enum SubmissionID {
    ByStoredID(u32),
    ByUsername(String),
}

pub async fn submit_scores(
    data: &Arc<RikaData>,
    osu_id: impl Into<SubmissionID>,
    mode: GameMode,
) -> CommandReturn {
    let RikaData {
        db,
        rosu,
        beatmap_cache,
        ..
    } = data.as_ref();

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

    #[derive(From)]
    enum BonkersferformanceAttributes {
        Osu(OsuPerformanceAttributes),
        Taiko(TaikoPerformanceAttributes),
        Mania(ManiaPerformanceAttributes),
    }

    let mut performance_information: Vec<(BonkersferformanceAttributes, (&Score, &u64))> = vec![];

    for (i, (score_id, score)) in new_scores.iter().enumerate() {
        let beatmap_file = beatmap_cache.get_beatmap_file(score.map_id).await?;
        let beatmap_rosu = rosu_pp::Beatmap::from_bytes(&beatmap_file).await?;

        macro_rules! calc {
            ($mode:ident) => {
                paste! {
                    [<$mode PP>]::new(&beatmap_rosu)
                        .mods(score.mods.into())
                        .n300(score.statistics.count_300 as usize)
                        .n100(score.statistics.count_100 as usize)
                        .n_misses(score.statistics.count_miss as usize)
                }
            };
        }

        let performance_attributes = match mode {
            GameMode::Osu => Some(
                calc!(Osu)
                    .n50(score.statistics.count_50 as usize)
                    .combo(score.max_combo as usize)
                    .calculate()
                    .into(),
            ),
            GameMode::Taiko => Some(
                calc!(Taiko)
                    .combo(score.max_combo as usize)
                    .calculate()
                    .into(),
            ),
            GameMode::Mania => Some(
                calc!(Mania)
                    .n320(score.statistics.count_geki as usize)
                    .n200(score.statistics.count_katu as usize)
                    .calculate()
                    .into(),
            ),
            _ => None,
        };

        if let Some(performance_attributes) = performance_attributes {
            performance_information.push((performance_attributes, (*score, score_id)));
            info!("Processed score number {} for {osu_id}", i + 1);
        }
    }

    let mut tx = db.begin().await?;

    // Can't we do all this in a single query at this point? i think so? i am not sure.
    // I mean, anyways this takes at most 30ms the chances of any deadlocks here are minimal.
    for (bonkers_performance, (score, score_id)) in performance_information {
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

        match bonkers_performance {
            BonkersferformanceAttributes::Osu(OsuPerformanceAttributes {
                pp,
                pp_acc,
                pp_aim,
                pp_flashlight,
                pp_speed,
                ..
            }) => {
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
            }
            BonkersferformanceAttributes::Taiko(TaikoPerformanceAttributes {
                pp,
                pp_acc,
                pp_difficulty,
                ..
            }) => sqlx::query!(
                "
                INSERT INTO taiko_performance (id, accuracy, difficulty, overall)
                VALUES (?, ?, ?, ?)
                ",
                score_id,
                pp_acc,
                pp_difficulty,
                pp
            ),
            BonkersferformanceAttributes::Mania(ManiaPerformanceAttributes {
                pp,
                pp_difficulty,
                ..
            }) => sqlx::query!(
                "
                INSERT INTO mania_performance (id, difficulty, overall)
                VALUES (?, ?, ?)
                ",
                score_id,
                pp_difficulty,
                pp
            ),
        }
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
                WHERE osu_user_id = ? AND mode = ?
                ORDER BY created_at DESC
                LIMIT 100
            ) as top_100
        ) AND osu_user_id = ? AND mode = ?
        ",
        &osu_id,
        &osu_id,
        &mode_bits,
        &mode_bits
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
