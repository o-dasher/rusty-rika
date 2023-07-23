use poise::command;

use crate::{commands::CommandReturn, RikaContext};

pub mod avatar;

use avatar::avatar;

#[command(slash_command, subcommands("avatar"))]
pub async fn user(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}
