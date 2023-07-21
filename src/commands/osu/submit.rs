use anyhow::anyhow;
use lexicon::t;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMode;
use tokio::sync::mpsc;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    error::RikaError,
    tasks::osu::submit::submit_scores,
    utils::{emojis::RikaMoji, replies::cool_text},
    RikaContext,
};

/// Submits your top plays, only works for STD.
#[poise::command(slash_command)]
pub async fn submit(ctx: RikaContext<'_>, mode: OsuMode) -> CommandReturn {
    let i18n = ctx.i18n();
    let (.., osu_id) = ctx.linked_osu_user().await?;

    let msg = ctx
        .say(cool_text(
            RikaMoji::ChocolateBar,
            &t!(i18n.osu.submit.too_long_warning),
        ))
        .await?;

    let (sender, mut receiver) = mpsc::channel(100);

    let submit_result = tokio::spawn(submit_scores(
        ctx.data().clone(),
        osu_id,
        GameMode::from(mode),
        sender.into(),
    ));

    while let Some((amount, out_of)) = receiver.recv().await {
        msg.edit(ctx, |b| {
            b.content(cool_text(
                RikaMoji::ChocolateBar,
                &t!(i18n.osu.submit.progress_shower).r((amount, out_of)),
            ))
        })
        .await?
    }

    if let Ok(result) = submit_result.await {
        result.map_err(|e| match e {
            RikaError::LockError(..) => {
                anyhow!(t!(i18n.osu.submit.already_submitting).clone()).into()
            }
            e => e,
        })?
    }

    msg.edit(ctx, |r| {
        r.content(cool_text(RikaMoji::Ok, &t!(i18n.osu.submit.submitted)))
    })
    .await?;

    Ok(())
}
