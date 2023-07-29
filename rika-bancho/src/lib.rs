pub mod commands;
pub mod error;

use std::vec;

use commands::owo::owo;

use error::handle_error;
use kani_kani::{BoxedError, KaniContext, KaniFramework};
use nasus::BanchoConfig;
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

pub struct RikaData {}
pub type RikaContext = KaniContext<RikaData>;

pub async fn run() -> Result<(), BoxedError> {
    let config = envy::prefixed("BANCHO_").from_env::<RikaConfig>()?;

    let bancho_config = BanchoConfig {
        host: config.host,
        port: config.port,
        bot: config.bot,
        username: config.username,
        irc_token: config.irc_token,
    };

    let kani_kani = KaniFramework {
        config: bancho_config,
        data: RikaData {},
        prefix: config.prefix,
        commands: vec![(vec!["owo"], &owo)],
        on_error: &handle_error,
    };

    kani_kani.run().await?;

    Ok(())
}
