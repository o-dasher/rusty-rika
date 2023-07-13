pub mod waifu;

use poise::command;
use waifu::waifu;

use crate::RikaContext;

use super::CommandReturn;

#[command(slash_command, subcommands("waifu"))]
pub async fn rate(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}
