pub mod link;
pub mod recommend;
pub mod submit;

use link::link;
use poise::{async_trait, command, ChoiceParameter};
use recommend::recommend;
use rosu_v2::prelude::GameMode;
use sqlx::Result;
use submit::submit;

use crate::{commands::CommandReturn, error, RikaContext, RikaData};

#[command(slash_command, subcommands("link", "submit", "recommend"))]
pub async fn osu(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}

#[derive(ChoiceParameter, Default, Clone, Copy)]
#[repr(u8)]
pub enum OsuMode {
    #[default]
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
}

impl From<OsuMode> for GameMode {
    fn from(val: OsuMode) -> Self {
        GameMode::from(val as u8)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RikaOsuError {
    #[error("You must link your account to use this command.")]
    NotLinked,

    #[error("You must submit some scores before using this command. Try `/osu submit`")]
    RequiresSubmission,

    #[error("This command does not support this mode.")]
    UnsupportedMode
}

#[async_trait]
pub trait RikaOsuContext {
    async fn linked_osu_user(&self) -> Result<((), u32), RikaOsuError>;
}

#[async_trait]
impl RikaOsuContext for RikaContext<'_> {
    async fn linked_osu_user(&self) -> Result<((), u32), RikaOsuError> {
        let RikaData { db, .. } = self.data().as_ref();

        let user = sqlx::query!(
            "SELECT * FROM rika_user WHERE discord_id=?",
            &self.author().id.to_string()
        )
        .fetch_one(db)
        .await
        .map_err(|_| RikaOsuError::NotLinked)?;

        let osu_id = user.osu_id.ok_or_else(|| RikaOsuError::NotLinked)?;

        Ok(((), osu_id))
    }
}
