use anyhow::anyhow;
use lexicon::t;
use roricon::RoriconTrait;

use crate::{
    commands::CommandReturn,
    utils::{emojis::RikaMoji, markdown::mono, replies::cool_text},
    RikaContext, RikaData,
};

#[poise::command(slash_command)]
pub async fn link(ctx: RikaContext<'_>, name: String) -> CommandReturn {
    let i18n = ctx.i18n();
    let RikaData { rosu, db, .. } = ctx.data();

    let osu_user = rosu
        .user(&name)
        .await
        .map_err(|_| anyhow!(t!(i18n.osu.link.failed).r(name.clone())))?;

    let osu_user_id = osu_user.user_id;

    let mut tx = db.begin().await?;

    sqlx::query!(
        "
        INSERT INTO rika_user (discord_id, osu_id)
        VALUES (?, ?)
        ON DUPLICATE KEY UPDATE
            osu_id = VALUES(osu_id)
        ",
        &ctx.author().id.to_string(),
        &osu_user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "
        INSERT IGNORE INTO osu_user (id)
        VALUES (?)
        ",
        &osu_user_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    ctx.say(cool_text(
        RikaMoji::Ok,
        &t!(i18n.osu.link.linked).r(mono(osu_user.username.to_string())),
    ))
    .await?;

    Ok(())
}
