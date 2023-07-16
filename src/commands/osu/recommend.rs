use roricon::RoriconTrait;
use rosu_pp::Mods;
use rosu_v2::prelude::GameMods;
use tuple_map::TupleMap4;
use anyhow::anyhow;

use crate::{
    commands::{osu::RikaOsuContext, CommandReturn},
    models::osu_score::OsuScore,
    RikaContext, RikaData,
};

#[derive(sqlx::FromRow)]
struct OsuPerformanceAverage {
    speed: Option<f64>,
    accuracy: Option<f64>,
    aim: Option<f64>,
    flashlight: Option<f64>,
}

#[poise::command(slash_command)]
pub async fn recommend(ctx: RikaContext<'_>) -> CommandReturn {
    let i18n = ctx.i18n();
    let RikaData { rosu, db, .. } = ctx.data();

    let (.., osu_id) = ctx.linked_osu_user().await?;

    let user_average = sqlx::query_as!(
        OsuPerformanceAverage,
        "
        SELECT
        AVG(pp.speed) as speed,
        AVG(pp.accuracy) as accuracy,
        AVG(pp.aim) as aim,
        AVG(pp.flashlight) as flashlight
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE osu_user_id = $1
        ",
        osu_id
    )
    .fetch_one(db)
    .await?;

    let (min_speed, min_accuracy, min_aim, min_flashlight) = (
        user_average.speed,
        user_average.accuracy,
        user_average.aim,
        user_average.flashlight,
    )
        .map(|x| x.unwrap_or_default() * 0.75);

    let possible_recommendation = sqlx::query_as!(
        OsuScore,
        "
        SELECT s.*
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        GROUP BY s.id
        HAVING
        s.osu_user_id != $1 AND
        AVG(pp.speed) BETWEEN $2 AND $3 AND
        AVG(pp.accuracy) BETWEEN $4 AND $5 AND
        AVG(pp.aim) BETWEEN $6 AND $7 AND
        AVG(pp.flashlight) BETWEEN $8 AND $9
        ORDER BY RANDOM()
        ",
        osu_id,
        min_speed,
        user_average.speed,
        min_accuracy,
        user_average.accuracy,
        min_aim,
        user_average.aim,
        min_flashlight,
        user_average.flashlight
    )
    .fetch_one(db)
    .await
    .map_err(|_| anyhow!("Iai parsa nao achei mapa pra ce nao"))?;

    ctx.say(format!(
        "SEGUINTE PARSA TE RECOMENDO JOGAR ISSO AQUI COM OS MOD {} TLG https://osu.ppy.sh/b/{}",
        GameMods::try_from(possible_recommendation.mods as u32)?,
        possible_recommendation.map_id
    ))
    .await?;

    Ok(())
}
