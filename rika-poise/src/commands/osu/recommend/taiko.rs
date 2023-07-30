use super::{get_weighter, mid_interval};
use crate::{
    commands::osu::recommend::query_recommendation,
    utils::{emojis::RikaMoji, markdown::mono, replies::cool_text},
};
use anyhow::anyhow;
use lexicon::t_prefix;
use paste::paste;
use rika_model::{rika_cord, SharedRika};
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMods;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    create_weighter, fetch_performance, init_recommendation,
    models::osu_score::TaikoPerformance,
    reply_recommendation,
};

#[poise::command(slash_command)]
pub async fn taiko(ctx: rika_cord::Context<'_>, range: Option<f32>) -> CommandReturn {
    let rika_cord::Data { shared, .. } = ctx.data().as_ref();
    let SharedRika { db, .. } = shared.as_ref();

    init_recommendation!($, db, ctx, range, Taiko);

    let (min_acc, max_acc) = apply_weight!(accuracy);
    let (min_diff, max_diff) = apply_weight!(difficulty);

    let recommendation = query_recommendation(
        db,
        "taiko",
        vec![
            ("accuracy", (min_acc, max_acc)),
            ("difficulty", (min_diff, max_diff)),
        ],
    );

    reply_recommendation!(ctx, recommendation);

    Ok(())
}
