use derive_more::From;
use itertools::Itertools;
use log::info;
use rosu_pp::{osu::OsuPerformanceAttributes, OsuPP};
use rosu_v2::prelude::GameMode;

use crate::{commands::CommandReturn, models::osu_score::OsuPerformance, RikaData};

#[derive(From)]
pub enum SubmissionID {
    ByStoredID(i64),
    ByUsername(String),
}

pub async fn submit_scores(
    RikaData {
        db,
        rosu,
        beatmap_cache,
        ..
    }: &RikaData,
    osu_id: impl Into<SubmissionID>,
) -> CommandReturn {
    let mode = GameMode::Osu;
    let mode_bits = mode as i16;

    let osu_id = match osu_id.into() {
        SubmissionID::ByStoredID(id) => id as u32,
        SubmissionID::ByUsername(username) => rosu.user(username).await?.user_id,
    };

    let osu_scores = rosu.user_scores(osu_id).limit(100).mode(mode).await?;

    let rika_osu_scores: Vec<OsuPerformance> = sqlx::query_as!(
        OsuPerformance,
        "
        SELECT pp.* FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE s.osu_user_id = $1 AND s.mode = $2
        ",
        osu_id as i16,
        mode_bits
    )
    .fetch_all(db)
    .await?;

    let mut tx = db.begin().await?;

    let new_scores = osu_scores
        .iter()
        .filter_map(|s| {
            s.score_id.and_then(|score_id| {
                let existing = rika_osu_scores.iter().find(|s| s.id as u64 == score_id);

                match existing {
                    Some(..) => None,
                    None => Some((score_id, s)),
                }
            })
        })
        .collect_vec();

    for (i, (score_id, score)) in new_scores.iter().enumerate() {
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

        let stored_score_id = *score_id as i64;

        sqlx::query!(
            "
            INSERT INTO osu_score (id, osu_user_id, map_id, mods, mode)
            VALUES ($1, $2, $3, $4, $5)
            ",
            stored_score_id,
            osu_id as i64,
            score.map_id as i32,
            score.mods.bits() as i32,
            mode_bits
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "
            INSERT INTO osu_performance (id, aim, speed, flashlight, accuracy, overall)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
            stored_score_id,
            pp_aim,
            pp_speed,
            pp_flashlight,
            pp_acc,
            pp
        )
        .execute(&mut *tx)
        .await?;

        info!("Processed score number {i} for {osu_id}");
    }

    tx.commit().await?;

    Ok(())
}
