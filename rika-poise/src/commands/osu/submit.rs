use rika_model::{
    barebone_commands::submit::{submit_barebones, SubmitStatus},
    rika_cord,
};
use roricon::RoriconTrait;
use tokio::sync::mpsc;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    utils::{emojis::RikaMoji, replies::cool_text},
};

/// Submits your top plays, only works for STD.
#[poise::command(slash_command)]
pub async fn submit(ctx: rika_cord::Context<'_>, mode: OsuMode) -> CommandReturn {
    let (.., osu_id) = ctx.linked_osu_user().await?;

    let (sender, mut receiver) = mpsc::unbounded_channel();

    let data = ctx.clone().data().clone().shared.clone();
    let i18n = ctx.clone().i18n().clone();

    tokio::spawn(
        submit_barebones(
            data,
            osu_id,
            i18n,
            sender,
            mode.into(),
        )
    );

    while let Some((status, text)) = receiver.recv().await {
        match status {
            SubmitStatus::Start(sender) => {
                sender.send(ctx.say(cool_text(RikaMoji::ChocolateBar, &text)).await?);
            }
            SubmitStatus::Sending((msg, ..)) => {
                msg.edit(ctx, |b| {
                    b.content(&cool_text(RikaMoji::ChocolateBar, &text))
                })
                    .await?;
            }
            SubmitStatus::Finished(msg) => {
                msg.edit(ctx, |b| {
                    b.content(&cool_text(RikaMoji::ChocolateBar, &text))
                });
            }
        };
    }

    Ok(())
}
