pub mod commands;
pub mod error;
pub mod models;
pub mod setup;
pub mod translations;
pub mod utils;

use std::sync::Arc;

use commands::{math::math, osu::osu, owner::owner, rate::rate, user::user};
use error::RikaError;
use lexicon::Localizer;
use log::error;

use poise::{futures_util::TryFutureExt, serenity_prelude::GatewayIntents, FrameworkOptions};
use rika_model::SharedRika;
use roricon::{apply_translations, RoriconMetaTrait};

use serde::Deserialize;
use setup::setup;
use translations::{pt_br::locale_pt_br, rika_localizer::RikaLocalizer, RikaLocale};

#[derive(Deserialize)]
pub struct RikaConfig {
    bot_token: String,
    development_guild: Option<u64>,
    scraped_country: String,
}

pub struct RikaData {
    pub config: RikaConfig,
    pub shared: Arc<SharedRika>,
    pub locales: Localizer<RikaLocale, RikaLocalizer>,
}

pub type RikaContext<'a> = poise::Context<'a, Arc<RikaData>, RikaError>;

impl<'a> RoriconMetaTrait<'a, RikaLocale, RikaLocalizer> for RikaContext<'a> {
    fn locales(&self) -> &'a Localizer<RikaLocale, RikaLocalizer> {
        &self.data().locales
    }
}

pub async fn run(
    shared_rika: Arc<SharedRika>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt().with_target(true).pretty().init();

    let config = envy::from_env::<RikaConfig>()?;

    let mut commands = vec![user(), owner(), math(), rate(), osu()];
    let locales = Localizer::new(vec![(RikaLocale::BrazilianPortuguese, locale_pt_br)]);

    apply_translations(&mut commands, &locales);

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands,
            on_error: |err| Box::pin(error::on_error(err).unwrap_or_else(|e| error!("{}", e))),
            ..Default::default()
        })
        .token(&config.bot_token)
        .intents(GatewayIntents::non_privileged())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move { setup(ctx, framework, locales, config, shared_rika).await })
        })
        .run()
        .await?;

    Ok(())
}
