use poise::command;
use rika_model::rika_cord;

use crate::commands::CommandReturn;

pub mod avatar;

use avatar::avatar;

#[command(slash_command, subcommands("avatar"))]
pub async fn user(_ctx: rika_cord::Context<'_>) -> CommandReturn {
    Ok(())
}
