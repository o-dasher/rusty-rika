use derive_more::From;
use log::error;
use poise::serenity_prelude;
use strum::Display;

use crate::RikaData;

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

            ctx.send(|b| b.content(error.to_string()).ephemeral(true))
                .await?;
        }
        e => poise::builtins::on_error(e)
            .await
            .unwrap_or_else(|e| error!("{}", e)),
    }

    Ok(())
}

