use anyhow::anyhow;
use lexicon::t;
use roricon::RoriconTrait;

use crate::{
    commands::CommandReturn,
    utils::{emojis::RikaMoji, replies::cool_text, markdown::mono},
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

    let osu_user_id = osu_user.user_id as i64;

    let mut tx = db.begin().await?;

    // CREATE USER
    let rika_user = sqlx::query!(
        "
        INSERT INTO rika_user (discord_id)
        VALUES ($1)
        ON CONFLICT (discord_id) DO UPDATE
        SET osu_id = $2
        RETURNING id
        ",
        &ctx.author().id.to_string(),
        &osu_user_id
    )
    .fetch_one(&mut *tx)
    .await?;

    let rika_user_id = rika_user.id as i64;

    // SET PREVIOUS USER TO NULL
    sqlx::query!(
        "
        UPDATE osu_user
        SET rika_user_id = NULL
        WHERE rika_user_id = $1
        ",
        &rika_user_id
    )
    .execute(&mut *tx)
    .await?;

    // INSERT OSU_USER IF NOT EXISTS
    sqlx::query!(
        "
        INSERT INTO osu_user (id, rika_user_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        ",
        &osu_user_id,
        &rika_user_id
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
