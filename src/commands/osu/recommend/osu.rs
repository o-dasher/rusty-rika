use super::{get_weighter, mid_interval};
use crate::utils::{emojis::RikaMoji, markdown::mono, replies::cool_text};
use anyhow::anyhow;
use lexicon::t_prefix;
use paste::paste;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMods;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext, RikaOsuError},
        CommandReturn,
    },
    create_weighter, fetch_performance, init_recommendation,
    models::osu_score::{OsuPerformance, OsuScore},
    reply_recommendation, RikaContext, RikaData,
};

#[poise::command(slash_command)]
pub async fn osu(ctx: RikaContext<'_>, range: Option<f32>) -> CommandReturn {
    let RikaData { db, .. } = ctx.data().as_ref();

    init_recommendation!($, ctx, range, Osu);

    let (min_speed, max_speed) = apply_weight!(speed);
    let (min_acc, max_acc) = apply_weight!(accuracy);
    let (min_aim, max_aim) = apply_weight!(aim);
    let (min_fl, max_fl) = apply_weight!(flashlight);

    let recommendation = sqlx::query_as!(
        OsuScore,
        "
        SELECT s.*
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE 
            pp.speed BETWEEN ? AND ? AND
            pp.accuracy BETWEEN ? AND ? AND
            pp.aim BETWEEN ? AND ? AND
            pp.flashlight BETWEEN ? AND ?
        ORDER BY RAND()
        ",
        min_speed,
        max_speed,
        min_acc,
        max_acc,
        min_aim,
        max_aim,
        min_fl,
        max_fl
    )
    .fetch_one(db)
    .await;

    reply_recommendation!(ctx, recommendation);

    Ok(())
}
