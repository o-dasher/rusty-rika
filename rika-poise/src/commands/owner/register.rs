use crate::{commands::CommandReturn, RikaContext};

/// Register Slash Commands (Owner Only)
#[poise::command(owners_only, slash_command)]
pub async fn register(ctx: RikaContext<'_>) -> CommandReturn {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
