#![deny(rust_2018_idioms)]

pub mod commands;
pub mod error;
pub mod models;
pub mod setup;
pub mod utils;

use std::sync::Arc;

use commands::{math::math, osu::osu, owner::owner, rate::rate, user::user};
use log::error;

use poise::{futures_util::TryFutureExt, serenity_prelude::GatewayIntents, FrameworkOptions};
use rika_model::{rika_cord, SharedRika};
use roricon::apply_translations;

use setup::setup;

pub async fn run(
    shared_rika: Arc<SharedRika>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().with_target(true).pretty().init();

    let config = envy::from_env::<rika_cord::Config>()?;

    let mut commands = vec![user(), owner(), math(), rate(), osu()];

    apply_translations(&mut commands, &shared_rika.locales);

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands,
            on_error: |err| Box::pin(error::on_error(err).unwrap_or_else(|e| error!("{}", e))),
            ..Default::default()
        })
        .token(&config.bot_token)
        .intents(GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move { setup(ctx, framework, config, shared_rika).await })
        })
        .run()
        .await?;

    Ok(())
}
