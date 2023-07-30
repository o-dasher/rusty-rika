pub mod link;
pub mod recommend;
pub mod submit;

use link::link;
use poise::{async_trait, command, ChoiceParameter};
use recommend::recommend;
use rika_model::{rika_cord, SharedRika};
use rosu_v2::prelude::GameMode;
use sqlx::Result;
use submit::submit;

use crate::commands::CommandReturn;

#[command(slash_command, subcommands("link", "submit", "recommend"))]
pub async fn osu(_ctx: rika_cord::Context<'_>) -> CommandReturn {
    Ok(())
}

#[derive(ChoiceParameter, Default, Clone, Copy)]
#[repr(u8)]
pub enum OsuMode {
    #[default]
    #[name = "osu"]
    Osu = 0,

    #[name = "taiko"]
    Taiko = 1,

    #[name = "catch"]
    Catch = 2,

    #[name = "mania"]
    Mania = 3,
}

impl From<OsuMode> for GameMode {
    fn from(val: OsuMode) -> Self {
        GameMode::from(val as u8)
    }
}

#[async_trait]
pub trait RikaOsuContext {
    async fn linked_osu_user(&self) -> Result<((), u32), rika_cord::OsuError>;
}

#[async_trait]
impl RikaOsuContext for rika_cord::Context<'_> {
    async fn linked_osu_user(&self) -> Result<((), u32), rika_cord::OsuError> {
        let rika_cord::Data { shared, .. } = self.data().as_ref();
        let SharedRika { db, .. } = shared.as_ref();

        let user = sqlx::query!(
            "SELECT * FROM rika_user WHERE discord_id=?",
            &self.author().id.to_string()
        )
        .fetch_one(db)
        .await
        .map_err(|_| rika_cord::OsuError::NotLinked)?;

        let osu_id = user.osu_id.ok_or(rika_cord::OsuError::NotLinked)?;

        Ok(((), osu_id))
    }
}
