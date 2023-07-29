use nasus::CmdOut;

use crate::{error::RikaBanchoError, RikaKaniContext};

pub async fn owo(
    RikaKaniContext { irc, sender, .. }: RikaKaniContext,
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
