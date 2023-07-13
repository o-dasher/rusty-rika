pub mod commands;
pub mod error;
pub mod translations;
pub mod utils;

use commands::{math::math, owner::owner, rate::rate, user::user};
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
use translations::{pt_br::locale_pt_br, rika_localizer::RikaLocalizer, RikaLocale};

#[derive(Deserialize)]
pub struct RikaConfig {
    bot_token: String,
    development_guild: u64,
}

pub struct RikaData {
    config: RikaConfig,
    locales: Localizer<RikaLocale, RikaLocalizer>,
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

    let mut commands = vec![user(), owner(), math(), rate()];
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
        .setup(move |ctx, _ready, _framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &vec![owner()],
                    GuildId(config.development_guild),
                )
                .await?;
                Ok(RikaData { config, locales })
            })
        })
        .run()
        .await
        .expect("Failed to run bot");
}
