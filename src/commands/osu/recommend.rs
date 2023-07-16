use anyhow::anyhow;
use num_traits::Float;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMods;
use tuple_map::TupleMap4;

use crate::{
    commands::{osu::RikaOsuContext, CommandReturn},
    RikaContext, RikaData,
};

#[poise::command(slash_command)]
pub async fn recommend(ctx: RikaContext<'_>) -> CommandReturn {
    let _i18n = ctx.i18n();
    let RikaData {  db, .. } = ctx.data();

    let (.., osu_id) = ctx.linked_osu_user().await?;

    let user_average = sqlx::query!(
        "
        SELECT
        AVG(pp.speed) as speed,
        AVG(pp.accuracy) as accuracy,
        AVG(pp.aim) as aim,
        AVG(pp.flashlight) as flashlight
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE osu_user_id = ?
        ",
        osu_id
    )
    .fetch_one(db)
    .await?;

    let (
        (min_speed, max_speed),
        (min_acc, max_acc),
        (min_aim, max_aim),
        (min_flashlight, max_flashlight),
    ) = (
        user_average.speed,
        user_average.accuracy,
        user_average.aim,
        user_average.flashlight,
    )
        .map(|x| {
            let x = x.unwrap_or_default();

            fn mid_interval<F: Float>(x: F, delta: F) -> (F, F) {
                let d = delta / F::from(2).unwrap();
                (x * (F::one() - d), x * (F::one() + d))
            }

            mid_interval(x, 0.5)
        });

    let possible_recommendation = sqlx::query!(
        "
        SELECT s.*
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        GROUP BY s.id
        HAVING
            s.osu_user_id != ? AND
            AVG(pp.speed) BETWEEN ? AND ? AND
            AVG(pp.accuracy) BETWEEN ? AND ? AND
            AVG(pp.aim) BETWEEN ? AND ? AND
            AVG(pp.flashlight) BETWEEN ? AND ?
        ORDER BY RAND()
        ",
        osu_id,
        min_speed,
        max_speed,
        min_acc,
        max_acc,
        min_aim,
        max_aim,
        min_flashlight,
        max_flashlight
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
