use crate::{commands::CommandReturn, RikaContext};

#[poise::command(slash_command)]
pub async fn calc(
    _ctx: RikaContext<'_>,
    #[description = "Selected expression"] _expression: String,
) -> CommandReturn {
    Ok(())
}
