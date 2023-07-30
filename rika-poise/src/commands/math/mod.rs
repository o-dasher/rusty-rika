use poise::command;
use rika_model::rika_cord;

pub mod calc;

use super::CommandReturn;
use calc::calc;

#[command(slash_command, subcommands("calc"))]
pub async fn math(_ctx: rika_cord::Context<'_>) -> CommandReturn {
    Ok(())
}
