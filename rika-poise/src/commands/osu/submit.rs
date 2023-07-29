use anyhow::anyhow;
use lexicon::t_prefix;
use rika_model::{
    osu::submit::{ScoreSubmitter, SubmissionError},
    SharedRika,
};
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMode;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    utils::{emojis::RikaMoji, replies::cool_text},
    RikaContext, RikaData,
};

/// Submits your top plays, only works for STD.
#[poise::command(slash_command)]
pub async fn submit(ctx: RikaContext<'_>, mode: OsuMode) -> CommandReturn {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.osu.submit);

    let (.., osu_id) = ctx.linked_osu_user().await?;

    let RikaData { shared, .. } = ctx.data().as_ref();
    let SharedRika {
        score_submitter, ..
    } = shared.as_ref();

    let msg = ctx
        .say(cool_text(RikaMoji::ChocolateBar, &t!(too_long_warning)))
        .await?;

    let (to_submit, mut receiver) = ScoreSubmitter::begin_submission(&score_submitter);

    let submit_result =
        tokio::spawn(async move { to_submit.submit_scores(osu_id, GameMode::from(mode)).await });

    while let Some((amount, out_of)) = receiver.recv().await {
        msg.edit(ctx, |b| {
            b.content(cool_text(
                RikaMoji::ChocolateBar,
                &t!(progress_shower).r((amount, out_of)),
            ))
        })
        .await?
    }

    if let Ok(result) = submit_result.await {
        result.map_err(|e| match e {
            SubmissionError::IdLocker(..) => anyhow!(t!(already_submitting).clone()).into(),
            e => e,
        })?
    }

    msg.edit(ctx, |r| r.content(cool_text(RikaMoji::Ok, &t!(submitted))))
        .await?;

    Ok(())
}
