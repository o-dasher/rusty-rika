use derive_more::From;
use kani_kani::KaniError;
use strum::Display;

#[derive(thiserror::Error, Debug, From, Display)]
pub enum RikaBanchoError {
    Fallthrough,
}

pub async fn handle_error(e: KaniError<RikaBanchoError>) -> Result<(), RikaBanchoError> {
    match e {
        KaniError::CommandError(_e) => {}
        _ => {}
    }
    Ok(())
}
