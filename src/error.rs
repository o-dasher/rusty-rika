use std::sync::Arc;

use crate::{
    commands::osu::RikaOsuError,
    utils::{emojis::RikaMoji, replies::cool_text},
    RikaData,
};
use derive_more::From;
use id_locked::IDLockerError;
use log::error;
use poise::serenity_prelude;
use rosu_v2::prelude::OsuError;
use strum::Display;

#[derive(Debug, From, Display)]
pub enum RikaError {
    Serenity(serenity_prelude::Error),

    Anyhow(anyhow::Error),
    Sqlx(sqlx::Error),
    Osu(OsuError),
    Rosu(rosu_pp::ParseError),
    RikaOsu(RikaOsuError),
    LockError(IDLockerError),

    Fallthrough,
}

pub async fn on_error(
    error: poise::FrameworkError<'_, Arc<RikaData>, RikaError>,
) -> Result<(), RikaError> {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            tracing::warn!("FrameworkCommand: {error}");

            let reply_error = |message: &str| {
                let content = cool_text(RikaMoji::X, message);

                ctx.send(|r| r.content(content).ephemeral(true))
            };

            macro_rules! handle {
                ($a:expr, $b:expr) => {{
                    error!("{:?}", $a);
                    reply_error(&$b.to_string()).await?;
                }};
                ($var:expr) => {
                    handle!($var, $var)
                };
            }

            match error {
                RikaError::Anyhow(e) => handle!(e),
                RikaError::RikaOsu(e) => handle!(e),
                e => handle!(
                    e,
                    "Something unexpected happened while executing this command."
                ),
            }
        }
        e => poise::builtins::on_error(e)
            .await
            .unwrap_or_else(|e| error!("{e:?}")),
    }

    Ok(())
}
