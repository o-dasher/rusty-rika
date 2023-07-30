use std::sync::Arc;

use derive_more::From;
use id_locked::IDLockerError;
use lexicon::Localizer;
use poise::serenity_prelude;
use roricon::RoriconMetaTrait;
use serde::Deserialize;
use strum::Display;

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

#[derive(Debug, From, Display)]
pub enum Error {
    Serenity(serenity_prelude::Error),

    Anyhow(anyhow::Error),
    Osu(rosu_v2::error::OsuError),
    Sqlx(sqlx::Error),
    Rosu(rosu_pp::ParseError),
    RikaOsu(OsuError),
    LockError(IDLockerError),
    Submission(SubmissionError),

    Fallthrough,
}

pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

impl<'a> RoriconMetaTrait<'a, RikaLocale, RikaLocalizer> for Context<'a> {
    fn locales(&self) -> &'a Localizer<RikaLocale, RikaLocalizer> {
        &self.data().shared.locales
    }
}
