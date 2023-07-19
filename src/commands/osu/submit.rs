use lexicon::t;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMode;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    tasks::osu::submit::submit_scores,
    utils::{emojis::RikaMoji, replies::cool_text},
    RikaContext,
};

/// Submits your top plays, only works for STD.
#[poise::command(slash_command)]
pub async fn submit(ctx: RikaContext<'_>, mode: OsuMode) -> CommandReturn {
    let i18n = ctx.i18n();
    let (.., osu_id) = ctx.linked_osu_user().await?;

    ctx.defer().await?;

    submit_scores(ctx.data(), osu_id, GameMode::from(mode)).await?;

    ctx.say(cool_text(RikaMoji::Ok, &t!(i18n.osu.submit.submitted)))
        .await?;

    Ok(())
}
