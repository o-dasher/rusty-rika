use std::sync::Arc;

use kani_kani::KaniContext;
use rika_model::barebone_commands::submit::{submit_barebones, SubmitAfter, SubmitStatus};
use rika_model::osu::submit::SubmittableMode;
use tokio::sync::mpsc;

use crate::{error::RikaBanchoError, KaniLocale, RikaData};

pub struct BanchoSubmitMode(SubmittableMode);

impl From<&str> for BanchoSubmitMode {
    fn from(value: &str) -> Self {
        Self(match value {
            "taiko" => SubmittableMode::Taiko,
            "mania" => SubmittableMode::Mania,
            _ => SubmittableMode::Osu,
        })
    }
}

impl From<Option<&String>> for BanchoSubmitMode {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(value) => value.as_str().into(),
            None => Self(SubmittableMode::Osu),
        }
    }
}

impl Default for BanchoSubmitMode {
    fn default() -> Self {
        Self(SubmittableMode::Osu)
    }
}

pub async fn submit(ctx: Arc<KaniContext<RikaData>>) -> Result<(), RikaBanchoError> {
    let KaniContext {
        args, data, sender, ..
    } = ctx.as_ref();
    let mode = BanchoSubmitMode::from(args.first());

    let (channel_sender, mut receiver) = mpsc::unbounded_channel();

    let submit_task = tokio::spawn(submit_barebones(
        data.shared.clone(),
        "Zaqqy".to_string(),
        ctx.i18n(),
        channel_sender,
        mode.0.into(),
    ));

    while let Some((status, text)) = receiver.recv().await {
        match status {
            SubmitStatus::Start => {
                ctx.say(&text)
                    .await
                    .map_err(|_| RikaBanchoError::Fallthrough)?;
            }
            SubmitStatus::After(after) => match after {
                SubmitAfter::Sending((amount,)) => {
                    if amount % 10 == 0 {
                        ctx.say(&text)
                            .await
                            .map_err(|_| RikaBanchoError::Fallthrough)?;
                    }
                }
                SubmitAfter::Finished => {
                    ctx.say(&text)
                        .await
                        .map_err(|_| RikaBanchoError::Fallthrough)?;
                }
            },
        };
    }

    if let Ok(task) = submit_task.await {
        task?;
    }

    Ok(())
}
