use anyhow::anyhow;
use lexicon::t_prefix;
use roricon::RoriconTrait;

use crate::{
    commands::CommandReturn,
    utils::markdown::{bold, mono},
    RikaContext,
};

#[poise::command(slash_command)]
pub async fn calc(
    ctx: RikaContext<'_>,
    #[description = "Selected expression"] expression: String,
) -> CommandReturn {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.math.calc);

    let display_expression = mono(&expression);

    let expression_result = exmex::eval_str::<f64>(&expression)
        .map_err(|_| anyhow!(t!(error_parse_fail).r(display_expression.clone())))?;

    let display_result = mono(expression_result.to_string());

    let response = t!(results_in).r((display_expression, display_result));

    ctx.say(bold(response)).await?;

    Ok(())
}
