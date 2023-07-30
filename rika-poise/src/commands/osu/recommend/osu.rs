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
    models::osu_score::OsuPerformance,
    reply_recommendation,
};

#[poise::command(slash_command)]
pub async fn osu(ctx: rika_cord::Context<'_>, range: Option<f32>) -> CommandReturn {
    let SharedRika { db, .. } = ctx.data().shared.as_ref();

    init_recommendation!($, db, ctx, range, Osu);

    let (min_speed, max_speed) = apply_weight!(speed);
    let (min_acc, max_acc) = apply_weight!(accuracy);
    let (min_aim, max_aim) = apply_weight!(aim);
    let (min_fl, max_fl) = apply_weight!(flashlight);

    let recommendation = query_recommendation(
        db,
        "osu",
        vec![
            ("speed", (min_speed, max_speed)),
            ("accuracy", (min_acc, max_acc)),
            ("aim", (min_aim, max_aim)),
            ("flashlight", (min_fl, max_fl)),
        ],
    );

    reply_recommendation!(ctx, recommendation);

    Ok(())
}
