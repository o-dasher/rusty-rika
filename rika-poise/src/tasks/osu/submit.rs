use std::{collections::HashSet, sync::Arc};

use derive_more::From;
use id_locked::IDLocker;
use itertools::Itertools;
use log::info;
use paste::paste;
use rosu_pp::{
    mania::ManiaPerformanceAttributes, osu::OsuPerformanceAttributes,
    taiko::TaikoPerformanceAttributes, ManiaPP, OsuPP, TaikoPP,
};
use rosu_v2::prelude::{GameMode, Score};
use sqlx::{MySql, QueryBuilder};
use strum::Display;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    RwLock,
};

use crate::{
    commands::{osu::RikaOsuError, CommandReturn},
    error::RikaError,
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

pub struct ScoreSubmitter {
    data: Option<Arc<RikaData>>,
    locker: IDLocker,
}

pub struct ReadyScoreSubmitter {
    submitter: Arc<RwLock<ScoreSubmitter>>,
    sender: Sender<(usize, usize)>,
}

impl Default for ScoreSubmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreSubmitter {
    pub fn new() -> Self {
        Self {
            data: None,
            locker: IDLocker::new(),
        }
    }

    pub fn provide_data(&mut self, data: Arc<RikaData>) {
        self.data = Some(data);
    }

    pub fn begin_submission(
        submitter: &Arc<RwLock<ScoreSubmitter>>,
    ) -> (ReadyScoreSubmitter, Receiver<(usize, usize)>) {
        let (sender, receiver) = mpsc::channel(100);

        (
            ReadyScoreSubmitter {
                submitter: submitter.clone(),
                sender,
            },
            receiver,
        )
    }
}

impl ReadyScoreSubmitter {
    pub async fn submit_scores(
        &self,
        osu_id: impl Into<SubmissionID>,
        mode: GameMode,
    ) -> CommandReturn {
        let submit_mode = match mode {
            GameMode::Osu => SubmittableMode::Osu,
            GameMode::Taiko => SubmittableMode::Taiko,
            GameMode::Mania => SubmittableMode::Mania,
            GameMode::Catch => Err(RikaOsuError::UnsupportedMode)?,
        };

        let submitter = self.submitter.read().await;

        let Some(data) = &submitter.data else {
            return Err(RikaError::Fallthrough)?
        };

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

        let locker_guard = submitter.locker.lock(osu_id.to_string())?;

        let osu_scores = rosu.user_scores(osu_id).limit(100).mode(mode).await?;

        #[derive(sqlx::FromRow)]
        struct ExistingScore {
            id: u64,
        }

        let rika_osu_scores: Vec<ExistingScore> = sqlx::query_as(&format!(
            "
            SELECT s.id FROM osu_score s
            JOIN {submit_mode}_performance pp ON s.id = pp.score_id
            WHERE s.osu_user_id = ?
            "
        ))
        .bind(osu_id)
        .fetch_all(db)
        .await?;

        let existing_scores: HashSet<_> = rika_osu_scores.into_iter().map(|s| s.id).collect();

        let new_scores = osu_scores
            .iter()
            .filter_map(|s| {
                s.score_id.and_then(|score_id| {
                    let is_new = !existing_scores.contains(&score_id);

                    is_new.then_some((score_id, s))
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

        let mut performance_information: Vec<(BonkersferformanceAttributes, (&Score, &u64))> =
            vec![];

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

                let _ = self.sender.send((display_index, new_scores.len())).await;

                info!("Processed score number {} for {osu_id}", display_index);
            }
        }

        let mut scores_query_builder = QueryBuilder::<MySql>::new(
            "
            INSERT INTO osu_score (id, osu_user_id, map_id, mods, mode)
            ",
        );

        scores_query_builder.push_values(
            &performance_information,
            |mut b, (.., (score, score_id))| {
                b.push_bind(score_id)
                    .push_bind(osu_id)
                    .push_bind(score.map_id)
                    .push_bind(score.mods.bits())
                    .push_bind(mode_bits);
            },
        );

        let base_pp_query = |to_insert: &str| {
            format!("INSERT INTO {submit_mode}_performance (score_id, overall, {to_insert})")
        };

        let base_performance_query = match submit_mode {
            SubmittableMode::Osu => base_pp_query("aim, speed, flashlight, accuracy"),
            SubmittableMode::Taiko => base_pp_query("accuracy, difficulty"),
            SubmittableMode::Mania => base_pp_query("difficulty"),
        };

        let mut performance_query_builder = QueryBuilder::<MySql>::new(base_performance_query);

        performance_query_builder.push_values(
            &performance_information,
            |mut b, (bonkers_performance, (.., score_id))| {
                b.push_bind(score_id);

                match bonkers_performance {
                    BonkersferformanceAttributes::Osu(OsuPerformanceAttributes {
                        pp,
                        pp_acc,
                        pp_aim,
                        pp_flashlight,
                        pp_speed,
                        ..
                    }) => b
                        .push_bind(pp)
                        .push_bind(pp_aim)
                        .push_bind(pp_speed)
                        .push_bind(pp_flashlight)
                        .push_bind(pp_acc),
                    BonkersferformanceAttributes::Taiko(TaikoPerformanceAttributes {
                        pp,
                        pp_acc,
                        pp_difficulty,
                        ..
                    }) => b.push_bind(pp).push_bind(pp_acc).push_bind(pp_difficulty),
                    BonkersferformanceAttributes::Mania(ManiaPerformanceAttributes {
                        pp,
                        pp_difficulty,
                        ..
                    }) => b.push_bind(pp).push_bind(pp_difficulty),
                };
            },
        );

        let mut tx = db.begin().await?;

        scores_query_builder.build().execute(&mut *tx).await?;
        performance_query_builder.build().execute(&mut *tx).await?;

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

        locker_guard.unlock()?;

        Ok(())
    }
}
