use poise::command;

pub mod calc;

use crate::RikaContext;

use super::CommandReturn;
use calc::calc;

#[command(slash_command, subcommands("calc"))]
pub async fn math(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}
