use crate::{commands::CommandReturn, RikaContext};

#[poise::command(slash_command)]
pub async fn calc(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}
