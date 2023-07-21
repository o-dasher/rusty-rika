pub mod commands;
pub mod error;
pub mod models;
pub mod setup;
pub mod tasks;
pub mod translations;
pub mod utils;

use std::sync::Arc;

use commands::{math::math, osu::osu, owner::owner, rate::rate, user::user};
use dotenvy::dotenv;
use error::RikaError;
use id_locked::IDLocker;
use lexicon::Localizer;
use log::error;

use poise::{futures_util::TryFutureExt, serenity_prelude::GatewayIntents, FrameworkOptions};
use roricon::{apply_translations, RoriconMetaTrait};

use serde::Deserialize;
use setup::setup;
use sqlx::MySqlPool;

use translations::{pt_br::locale_pt_br, rika_localizer::RikaLocalizer, RikaLocale};
use utils::osu::BeatmapCache;

#[derive(Deserialize)]
pub struct RikaConfig {
    bot_token: String,
    development_guild: Option<u64>,
    osu_client_id: u64,
    osu_client_secret: String,
    database_url: String,
    scraped_country: String,
}

pub struct RikaData {
    pub config: RikaConfig,
    pub locales: Localizer<RikaLocale, RikaLocalizer>,
    pub rosu: rosu_v2::Osu,
    pub beatmap_cache: BeatmapCache,
    pub submit_locker: IDLocker,
    pub db: MySqlPool,
}

pub type RikaContext<'a> = poise::Context<'a, Arc<RikaData>, RikaError>;

impl<'a> RoriconMetaTrait<'a, RikaLocale, RikaLocalizer> for RikaContext<'a> {
    fn locales(&self) -> &'a Localizer<RikaLocale, RikaLocalizer> {
        &self.data().locales
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt().with_target(true).pretty().init();

    let config = envy::from_env::<RikaConfig>().expect("Environment variables must be set");

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
            Box::pin(async move { setup(ctx, framework, locales, config).await })
        })
        .run()
        .await
        .expect("Failed to run bot");
}
