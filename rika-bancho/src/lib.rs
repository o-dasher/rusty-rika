pub mod kani;

use std::vec;

use kani::{KaniContext, KaniFramework, KaniResult, WorkaroundError};
use nasus::{BanchoConfig, CmdOut};
use serde::Deserialize;

#[derive(Deserialize)]
struct RikaBanchoConfig {
    host: String,
    port: u16,
    bot: bool,
    username: String,
    irc_token: String,
    prefix: String,
}

pub struct KaniCommand();
pub struct RikaKaniData {}

pub type RikaKaniContext = KaniContext<RikaKaniData>;

async fn owo(RikaKaniContext { irc, sender, .. }: RikaKaniContext) -> KaniResult {
    irc.lock()
        .await
        .write_command(CmdOut::SendPM {
            receiver: sender,
            message: "OWO".to_string(),
        })
        .await
        .map_err(|_| WorkaroundError::Fucked)?;

    Ok(())
}

pub async fn run() -> KaniResult {
    let config = envy::prefixed("BANCHO_").from_env::<RikaBanchoConfig>()?;

    let bancho_config = BanchoConfig {
        host: config.host,
        port: config.port,
        bot: config.bot,
        username: config.username,
        irc_token: config.irc_token,
    };

    let kani_kani = KaniFramework {
        config: bancho_config,
        data: RikaKaniData {},
        prefix: config.prefix,
        commands: vec![(vec!["owo"], &owo)],
    };

    kani_kani.run().await?;

    Ok(())
}
