use kani_kani::KaniContext;
use nasus::CmdOut;

use crate::{error::RikaBanchoError, RikaData};

pub async fn submit(
    KaniContext {
        irc, sender, data, ..
    }: KaniContext<RikaData>,
) -> Result<(), RikaBanchoError> {
    irc.lock()
        .await
        .write_command(CmdOut::SendPM {
            message: "OWO".to_string(),
            receiver: sender,
        })
        .await
        .map_err(|_| RikaBanchoError::Fallthrough)?;

    Ok(())
}
