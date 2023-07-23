pub mod register;

use poise::command;
use register::register;

use crate::{commands::CommandReturn, RikaContext};

#[command(slash_command, subcommands("register"))]
pub async fn owner(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}
