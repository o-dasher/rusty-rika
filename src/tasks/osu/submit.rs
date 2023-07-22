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
use strum::Display;
use tokio::sync::mpsc::Sender;

use crate::{
    commands::{osu::RikaOsuError, CommandReturn},
    RikaData,
};

#[derive(From)]
pub enum SubmissionID {
    ByStoredID(u32),
    ByUsername(String),
}

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
enum SubmittableMode {
    Osu,
    Taiko,
    Mania,
}

// The buffer for the receiver of the messages here will always remains to be 100
// I clearly need to redesign how this is supposed to be actually done.

pub async fn submit_scores(
    data: Arc<RikaData>,
    osu_id: impl Into<SubmissionID>,
    mode: GameMode,
    sender: Option<Sender<(usize, usize)>>,
) -> CommandReturn {
    let submit_mode = match mode {
        GameMode::Osu => SubmittableMode::Osu,
        GameMode::Taiko => SubmittableMode::Taiko,
        GameMode::Mania => SubmittableMode::Mania,
        GameMode::Catch => Err(RikaOsuError::UnsupportedMode)?,
    };

    let RikaData {
        db,
        rosu,
        beatmap_cache,
        submit_locker,
        ..
    } = data.as_ref();

    let mode_bits = mode as i16;

    let osu_id = match osu_id.into() {
        SubmissionID::ByStoredID(id) => id,
        SubmissionID::ByUsername(username) => rosu.user(username).await?.user_id,
    };

    let locker_guard = submit_locker.lock(osu_id.to_string()).await?;

    let osu_scores = rosu.user_scores(osu_id).limit(100).mode(mode).await?;

    #[derive(sqlx::FromRow)]
    struct ExistingScore {
        osu_score_id: u64,
    }

    let rika_osu_scores: Vec<ExistingScore> = sqlx::query_as(&format!(
        "
        SELECT s.osu_score_id FROM osu_score s
        JOIN {submit_mode}_performance pp ON s.id = pp.id
        WHERE s.osu_user_id = ? AND s.mode = ?
        "
    ))
    .bind(osu_id)
    .bind(mode_bits)
    .fetch_all(db)
    .await?;

    let existing_scores: HashSet<_> = rika_osu_scores
        .into_iter()
        .map(|s| s.osu_score_id)
        .collect();

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
                }
                .mods(score.mods.into())
                .n300(calc!(+count_300))
                .n100(calc!(+count_100))
                .n_misses(calc!(+count_miss))
            };
            (-$dep:ident) => {
                score.$dep as usize
            };
            (+$dep:ident) => {
                score.statistics.$dep as usize
            };
        }

        let performance_attributes = match mode {
            GameMode::Osu => Some(
                calc!(Osu)
                    .n50(calc!(+count_50))
                    .combo(calc!(-max_combo))
                    .calculate()
                    .into(),
            ),
            GameMode::Taiko => Some(calc!(Taiko).combo(calc!(-max_combo)).calculate().into()),
            GameMode::Mania => Some(
                calc!(Mania)
                    .n320(calc!(+count_geki))
                    .n200(calc!(+count_katu))
                    .calculate()
                    .into(),
            ),
            _ => None,
        };

        if let Some(performance_attributes) = performance_attributes {
            performance_information.push((performance_attributes, (*score, score_id)));

            let display_index = i + 1;

            if let Some(s) = &sender {
                let _ = s.send((display_index, new_scores.len())).await;
            }

            info!("Processed score number {} for {osu_id}", display_index);
        }
    }

    let mut tx = db.begin().await?;

    // Can't we do all this in a single query at this point? i think so? i am not sure.
    // I mean, anyways this takes at most 30ms the chances of any deadlocks here are minimal.
    for (bonkers_performance, (score, score_id)) in performance_information {
        let inserted_score = sqlx::query(
            "
            INSERT INTO osu_score (osu_score_id, osu_user_id, map_id, mods, mode)
            VALUES (?, ?, ?, ?, ?)
            ",
        )
        .bind(score_id)
        .bind(osu_id)
        .bind(score.map_id)
        .bind(score.mods.bits())
        .bind(mode_bits)
        .execute(&mut *tx)
        .await?;

        let db_score_id = inserted_score.last_insert_id();

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
                    db_score_id,
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
                db_score_id,
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
                db_score_id,
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

    locker_guard.unlock().await?;

    Ok(())
}
