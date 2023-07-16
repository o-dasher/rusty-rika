pub mod link;
pub mod recommend;
pub mod submit;

use link::link;
use recommend::recommend;
use poise::{async_trait, command, ChoiceParameter};
use rosu_v2::prelude::GameMode;
use sqlx::Result;
use submit::submit;

use crate::{commands::CommandReturn, models::rika_user::RikaUser, RikaContext, RikaData};

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
    #[error("You must link your account to use this command")]
    NotLinked,
}

#[async_trait]
pub trait RikaOsuContext {
    async fn linked_osu_user(&self) -> Result<(RikaUser, i64), RikaOsuError>;
}

#[async_trait]
impl RikaOsuContext for RikaContext<'_> {
    async fn linked_osu_user(&self) -> Result<(RikaUser, i64), RikaOsuError> {
        let RikaData { db, .. } = self.data();

        let rika_user: RikaUser = sqlx::query_as!(
            RikaUser,
            "SELECT * FROM rika_user WHERE discord_id=$1",
            &self.author().id.to_string()
        )
        .fetch_one(db)
        .await
        .map_err(|_| RikaOsuError::NotLinked)?;

        let osu_id = rika_user.osu_id.ok_or_else(|| RikaOsuError::NotLinked)?;

        Ok((rika_user, osu_id))
    }
}
