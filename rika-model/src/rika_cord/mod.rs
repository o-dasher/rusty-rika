use std::sync::Arc;

use derive_more::From;
use id_locked::IDLockerError;
use lexicon::Localizer;
use poise::serenity_prelude;
use roricon::RoriconMetaTrait;
use serde::Deserialize;

use crate::{
    i18n::{rika_localizer::RikaLocalizer, RikaLocale},
    osu::submit::SubmissionError,
    SharedRika,
};

#[derive(Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub scraped_country: String,
    pub development_guild: Option<u64>,
}

pub struct Data {
    pub config: Config,
    pub shared: Arc<SharedRika>,
}

#[derive(thiserror::Error, Debug)]
pub enum OsuError {
    #[error("You must link your account to use this command.")]
    NotLinked,

    #[error("You must submit some scores before using this command. Try `/osu submit`")]
    RequiresSubmission,

    #[error("This command does not support this mode.")]
    UnsupportedMode,
}

#[derive(thiserror::Error, Debug, From)]
pub enum Error {
    #[error(transparent)]
    Serenity(serenity_prelude::Error),

    #[error(transparent)]
    Anyhow(anyhow::Error),

    #[error(transparent)]
    Osu(rosu_v2::error::OsuError),

    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Rosu(rosu_pp::ParseError),

    #[error(transparent)]
    RikaOsu(OsuError),

    #[error(transparent)]
    LockError(IDLockerError),

    #[error(transparent)]
    Submission(SubmissionError),

    #[error("Fallthrough")]
    Fallthrough,
}

pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

impl<'a> RoriconMetaTrait<RikaLocale, RikaLocalizer> for Context<'a> {
    fn locales(&self) -> &Localizer<RikaLocale, RikaLocalizer> {
        &self.data().shared.locales
    }
}
