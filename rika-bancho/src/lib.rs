pub mod commands;
pub mod error;

use std::{sync::Arc, vec};

use commands::submit::submit;
use error::handle_error;
use kani_kani::{BoxedError, KaniContext, KaniFramework};
use lexicon::{LocaleAccess, Localizer};
use nasus::BanchoConfig;
use rika_model::{
    i18n::{rika_localizer::RikaLocalizer, RikaLocale},
    SharedRika,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct RikaConfig {
    host: String,
    port: u16,
    bot: bool,
    username: String,
    irc_token: String,
    prefix: String,
}

pub struct RikaData {
    shared: Arc<SharedRika>,
}

pub type RikaContext = KaniContext<RikaData>;

pub async fn run(shared: Arc<SharedRika>) -> Result<(), BoxedError> {
    let config = envy::prefixed("BANCHO_").from_env::<RikaConfig>()?;

    let bancho_config = BanchoConfig {
        host: config.host,
        port: config.port,
        bot: config.bot,
        username: config.username,
        irc_token: config.irc_token,
    };

    let data = RikaData { shared };

    let kani_kani = KaniFramework {
        config: bancho_config,
        data,
        prefix: config.prefix,
        commands: vec![(vec!["submit"], &submit)],
        on_error: &handle_error,
    };

    kani_kani.run().await?;

    Ok(())
}

pub trait KaniLocale {
    fn i18n(&self) -> LocaleAccess<Localizer<RikaLocale, RikaLocalizer>>;
}

impl KaniLocale for RikaContext {
    fn i18n(&self) -> LocaleAccess<Localizer<RikaLocale, RikaLocalizer>> {
        self.data
            .shared
            .locales
            .get(RikaLocale::UnitedStatesEnglish)
    }
}
