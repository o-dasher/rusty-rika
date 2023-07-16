use lexicon::t;
use log::info;
use roricon::RoriconTrait;
use rosu_pp::{osu::OsuPerformanceAttributes, OsuPP};
use thiserror::Error;

use crate::{
    commands::{osu::OsuMode, CommandReturn},
    utils::{emojis::RikaMoji, replies::cool_text},
    RikaContext, RikaData,
};

#[derive(Default, sqlx::FromRow, Clone)]
pub struct OsuPerformance {
    pub id: i64,
    pub aim: f64,
    pub speed: f64,
    pub accuracy: f64,
    pub flashlight: f64,
    pub overall: f64,
}

#[derive(Error, Debug)]
pub enum SubmissionError {
    #[error("You must link your account to use this command.")]
    NotLinked,
}

#[poise::command(slash_command)]
pub async fn submit(ctx: RikaContext<'_>, mode: Option<OsuMode>) -> CommandReturn {
    let i18n = ctx.i18n();
    let RikaData {
        db,
        rosu,
        beatmap_cache,
        ..
    } = ctx.data();
    let mode_bits = mode.unwrap_or_default() as i16;

    ctx.defer().await?;

    let osu_id = sqlx::query!(
        "SELECT * FROM rika_user WHERE discord_id=$1",
        &ctx.author().id.to_string()
    )
    .fetch_one(db)
    .await
    .and_then(|u| Ok(u.osu_id))
    .map_err(|_| SubmissionError::NotLinked)?
    .ok_or_else(|| SubmissionError::NotLinked)?;

    let osu_scores = rosu.user_scores(osu_id as u32).limit(100).await?;
    let rika_osu_scores: Vec<OsuPerformance> = sqlx::query_as!(
        OsuPerformance,
        "
        SELECT pp.* FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE s.osu_user_id = $1 AND s.mode = $2
        ",
        osu_id,
        mode_bits
    )
    .fetch_all(db)
    .await?;

    let mut tx = db.begin().await?;

    for score in osu_scores {
        let Some(score_id) = score.score_id else {
            continue;
        };

        let existing_score = rika_osu_scores.iter().find(|s| s.id as u64 == score_id);

        if let Some(..) = existing_score {
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

        let stored_score_id = score_id as i64;

        sqlx::query!(
            "
            INSERT INTO osu_score (id, osu_user_id, mode)
            VALUES ($1, $2, $3)
            ",
            stored_score_id,
            osu_id,
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

        info!("Processed another score for {osu_id}");
    }

    tx.commit().await?;

    ctx.say(cool_text(RikaMoji::Ok, &t!(i18n.osu.submit.submitted)))
        .await?;

    Ok(())
}
