pub mod register;

use poise::command;
use register::register;
use rika_model::rika_cord;

use crate::commands::CommandReturn;

#[command(slash_command, subcommands("register"))]
pub async fn owner(_ctx: rika_cord::Context<'_>) -> CommandReturn {
    Ok(())
}
