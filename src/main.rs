pub mod commands;
pub mod error;
pub mod models;
pub mod tasks;
pub mod translations;
pub mod utils;

use commands::{math::math, osu::osu, owner::owner, rate::rate, user::user};
use dotenvy::dotenv;
use error::RikaError;
use lexicon::Localizer;
use log::error;
use poise::{
    futures_util::TryFutureExt,
    serenity_prelude::{GatewayIntents, GuildId},
    FrameworkOptions,
};
use roricon::{apply_translations, RoriconMetaTrait};
use serde::Deserialize;
use sqlx::{MySqlPool, pool::PoolOptions};
use translations::{pt_br::locale_pt_br, rika_localizer::RikaLocalizer, RikaLocale};
use utils::osu::BeatmapCache;

#[derive(Deserialize)]
pub struct RikaConfig {
    bot_token: String,
    development_guild: u64,
    osu_client_id: u64,
    osu_client_secret: String,
    database_url: String,
}

pub struct RikaData {
    pub config: RikaConfig,
    pub locales: Localizer<RikaLocale, RikaLocalizer>,
    pub rosu: rosu_v2::Osu,
    pub beatmap_cache: BeatmapCache,
    pub db: MySqlPool,
}

pub type RikaContext<'a> = poise::Context<'a, RikaData, RikaError>;

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
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId(config.development_guild),
                )
                .await?;

                let rosu = rosu_v2::Osu::builder()
                    .client_id(config.osu_client_id)
                    .client_secret(&config.osu_client_secret)
                    .build()
                    .await
                    .expect("Failed to connect to osu! api");

                let db = PoolOptions::new()
                    .max_connections(10)
                    .connect(&config.database_url)
                    .await
                    .expect("Failed to connect to database!");

                let beatmap_cache = BeatmapCache::new();

                let rika_data = RikaData {
                    config,
                    locales,
                    rosu,
                    beatmap_cache,
                    db,
                };

                Ok(rika_data)
            })
        })
        .run()
        .await
        .expect("Failed to run bot");
}
