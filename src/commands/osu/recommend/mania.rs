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
    models::osu_score::{ManiaPerformance, OsuScore},
    reply_recommendation, RikaContext, RikaData,
};

#[poise::command(slash_command)]
pub async fn mania(ctx: RikaContext<'_>, range: Option<f32>) -> CommandReturn {
    let RikaData { db, .. } = ctx.data().as_ref();

    init_recommendation!($, ctx, range, Mania);

    let (min_diff, max_diff) = apply_weight!(difficulty);

    let recommendation = sqlx::query_as!(
        OsuScore,
        "
        SELECT s.*
        FROM osu_score s
        JOIN mania_performance pp ON s.id = pp.id
        WHERE 
            pp.difficulty BETWEEN ? AND ?
        ORDER BY RAND()
        ",
        min_diff,
        max_diff
    )
    .fetch_one(db)
    .await;

    reply_recommendation!(ctx, recommendation);

    Ok(())
}
