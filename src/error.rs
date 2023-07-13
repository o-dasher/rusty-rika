use derive_more::From;
use log::error;
use poise::serenity_prelude::{self, MessageBuilder};
use strum::Display;

use crate::{utils::emojis::RikaMoji, RikaData};

#[derive(Debug, From, Display)]
pub enum RikaError {
    Serenity(serenity_prelude::Error),

    Anyhow(anyhow::Error),

    Fallthrough,
}

pub async fn on_error(
    error: poise::FrameworkError<'_, RikaData, RikaError>,
) -> Result<(), RikaError> {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            tracing::warn!("FrameworkCommand: {error}");

            let reply_error = |message: &str| {
                let content = MessageBuilder::new()
                    .push_bold(format!("{} | {}", RikaMoji::X, message))
                    .build();

                ctx.send(|r| r.content(content).ephemeral(true))
            };

            match error {
                RikaError::Anyhow(e) => {
                    reply_error(&e.to_string()).await?;
                }
                e => {
                    error!("{}", e);
                    reply_error("Something unexpected happened while executing this command...")
                        .await?;
                }
            }
        }
        e => poise::builtins::on_error(e)
            .await
            .unwrap_or_else(|e| error!("{}", e)),
    }

    Ok(())
}
