pub mod commands;
pub mod translations;
pub mod utils;

use commands::{owner::owner, user::user};
use derive_more::From;
use dotenvy::dotenv;
use lexicon::{Localizer};
use poise::{
    serenity_prelude::{self, GatewayIntents, GuildId},
    FrameworkOptions,
};
use roricon::{apply_translations, RoriconMetaTrait};
use serde::Deserialize;
use strum::Display;
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

#[derive(From, Debug, Display)]
pub enum RikaError {
    Serenity(serenity_prelude::Error),
    Anyhow(anyhow::Error),
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

    let config = envy::from_env::<RikaConfig>().expect("Environment variables must be set");
    let mut commands = vec![user(), owner()];

    let locales = Localizer::new(vec![(RikaLocale::BrazilianPortuguese, locale_pt_br)]);

    apply_translations(&mut commands, &locales);

    poise::Framework::builder()
        .options(FrameworkOptions {
            commands,
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
