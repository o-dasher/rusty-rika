use nasus::CmdOut;

use crate::{error::RikaBanchoError, RikaContext};

pub async fn owo(RikaContext { irc, sender, .. }: RikaContext) -> Result<(), RikaBanchoError> {
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
