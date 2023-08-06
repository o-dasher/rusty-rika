use crate::RikaData;
use derive_more::From;
use kani_kani::KaniError;
use rika_model::osu::submit::SubmissionError;
use strum::Display;

#[derive(thiserror::Error, Debug, From, Display)]
pub enum RikaBanchoError {
    MissingArguments,
    Fallthrough,
    SubmissionError(SubmissionError),
    Anyhow(anyhow::Error),
}

pub async fn handle_error(e: KaniError<RikaData, RikaBanchoError>) -> Result<(), RikaBanchoError> {
    match e {
        KaniError::CommandError(_e, ctx) => {
            ctx.say("Something something.")
                .await
                .map_err(|_| RikaBanchoError::Fallthrough)?;
        }
        _ => {}
    }

    Ok(())
}
