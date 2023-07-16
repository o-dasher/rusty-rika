pub mod link;
pub mod submit;

use link::link;
use poise::{command, ChoiceParameter};
use rosu_v2::prelude::GameMode;
use submit::submit;

use crate::{commands::CommandReturn, RikaContext};

#[command(slash_command, subcommands("link", "submit"))]
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
