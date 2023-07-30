use rika_model::rika_cord;

use crate::commands::CommandReturn;

/// Register Slash Commands (Owner Only)
#[poise::command(owners_only, slash_command)]
pub async fn register(ctx: rika_cord::Context<'_>) -> CommandReturn {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
