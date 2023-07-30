use std::sync::Arc;

use crate::utils::{emojis::RikaMoji, replies::cool_text};
use log::error;
use rika_model::rika_cord;

pub async fn on_error(
    error: poise::FrameworkError<'_, Arc<rika_cord::Data>, rika_cord::Error>,
) -> Result<(), rika_cord::Error> {
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
                rika_cord::Error::Anyhow(e) => handle!(e),
                rika_cord::Error::RikaOsu(e) => handle!(e),
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
