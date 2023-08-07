use rika_model::barebone_commands::submit::SubmitAfter;
use rika_model::rika_cord::Error;
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

    let submit_task = tokio::spawn(submit_barebones(
        ctx.data().shared.clone(),
        osu_id,
        ctx.i18n(),
        sender,
        mode.into(),
    ));

    let mut msg = None;

    while let Some((status, text)) = receiver.recv().await {
        match status {
            SubmitStatus::Start => {
                msg = Some(ctx.say(cool_text(RikaMoji::ChocolateBar, &text)).await?);
            }
            SubmitStatus::After(after) => {
                let msg = msg.clone().ok_or(Error::Fallthrough)?;

                match after {
                    SubmitAfter::Sending((..)) => {
                        msg.edit(ctx, |b| {
                            b.content(&cool_text(RikaMoji::ChocolateBar, &text))
                        })
                        .await?;
                    }
                    SubmitAfter::Finished => {
                        msg.edit(ctx, |b| {
                            b.content(&cool_text(RikaMoji::ChocolateBar, &text))
                        })
                        .await?;
                    }
                }
            }
        };
    }

    if let Ok(task) = submit_task.await {
        task?
    }

    Ok(())
}
