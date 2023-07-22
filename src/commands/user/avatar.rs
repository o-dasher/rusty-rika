use crate::{commands::CommandReturn, utils::emojis::RikaMoji, RikaContext};
use lexicon::t_prefix;
use poise::serenity_prelude::{self, Color};
use roricon::RoriconTrait;

#[poise::command(slash_command)]
pub async fn avatar(
    ctx: RikaContext<'_>,
    #[description = "Selected user"] user: Option<serenity_prelude::User>,
) -> CommandReturn {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.user.avatar);

    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let avatar = u.avatar_url().unwrap_or_else(|| u.default_avatar_url());

    let footer = if u == ctx.author() {
        t!(footer.eq).clone()
    } else {
        t!(footer.other).r(u.name.clone())
    };

    ctx.send(|r| {
        r.embed(|e| {
            e.image(avatar)
                .color(Color::PURPLE)
                .title(&format!("{} {}", RikaMoji::Art, &u.name))
                .footer(|f| f.text(footer))
        })
    })
    .await?;

    Ok(())
}
