use super::{get_weighter, mid_interval};
use crate::{
    commands::osu::recommend::query_recommendation,
    utils::{emojis::RikaMoji, markdown::mono, replies::cool_text},
};
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
    models::osu_score::ManiaPerformance,
    reply_recommendation, RikaContext, RikaData,
};

#[poise::command(slash_command)]
pub async fn mania(ctx: RikaContext<'_>, range: Option<f32>) -> CommandReturn {
    let RikaData { db, .. } = ctx.data().as_ref();

    init_recommendation!($, ctx, range, Mania);

    let (min_diff, max_diff) = apply_weight!(difficulty);

    let recommendation =
        query_recommendation(db, "mania", vec![("difficulty", (min_diff, max_diff))]);

    reply_recommendation!(ctx, recommendation);

    Ok(())
}
