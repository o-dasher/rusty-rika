use derive_more::From;
use log::error;
use poise::serenity_prelude::{self, MessageBuilder};
use strum::Display;

use crate::{utils::emojis::RikaMoji, RikaData};

#[derive(Debug, From, Display)]
pub enum RikaError {
    Serenity(serenity_prelude::Error),

    Anyhow(anyhow::Error),
}

pub async fn on_error(
    error: poise::FrameworkError<'_, RikaData, RikaError>,
) -> Result<(), RikaError> {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            tracing::warn!("FrameworkCommand: {error}");

            match error {
                RikaError::Anyhow(e) => {
                    let content = MessageBuilder::new()
                        .push_bold(format!("{} | {}", RikaMoji::X, e.to_string()))
                        .build();

                    ctx.send(|b| b.content(content).ephemeral(true)).await?;
                }
                e => error!("{}", e),
            }
        }
        e => poise::builtins::on_error(e)
            .await
            .unwrap_or_else(|e| error!("{}", e)),
    }

    Ok(())
}
