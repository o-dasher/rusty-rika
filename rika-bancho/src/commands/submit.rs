use anyhow::anyhow;
use kani_kani::KaniContext;
use lexicon::t_prefix;
use rika_model::{
    osu::submit::{ScoreSubmitter, SubmissionError, SubmittableMode},
    SharedRika,
};

use crate::{error::RikaBanchoError, KaniLocale, RikaData};

pub struct BanchoSubmitMode(SubmittableMode);

impl From<&str> for BanchoSubmitMode {
    fn from(value: &str) -> Self {
        Self(match value {
            "taiko" => SubmittableMode::Taiko,
            "mania" => SubmittableMode::Mania,
            _ => SubmittableMode::Osu,
        })
    }
}

impl From<Option<&String>> for BanchoSubmitMode {
    fn from(value: Option<&String>) -> Self {
        match value {
            Some(value) => value.as_str().into(),
            None => Self(SubmittableMode::Osu),
        }
    }
}

impl Default for BanchoSubmitMode {
    fn default() -> Self {
        Self(SubmittableMode::Osu)
    }
}

pub async fn submit(ctx: KaniContext<RikaData>) -> Result<(), RikaBanchoError> {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.osu.submit);

    let KaniContext {
        args, data, sender, ..
    } = &ctx;
    let RikaData { shared } = data.as_ref();
    let SharedRika {
        score_submitter, ..
    } = shared.as_ref();

    let mode: BanchoSubmitMode = args.first().into();

    ctx.say(&t!(too_long_warning))
        .await
        .map_err(|_| RikaBanchoError::Fallthrough)?;

    let sender_clone = sender.clone();

    let (to_submit, mut receiver) = ScoreSubmitter::begin_submission(&score_submitter);
    let submit_result =
        tokio::spawn(async move { to_submit.submit_scores(sender_clone, mode.0.into()).await });

    while let Some((amount, out_of)) = receiver.recv().await {
        if amount % 10 == 0 {
            ctx.say(&t!(progress_shower).r((amount, out_of)))
                .await
                .map_err(|_| RikaBanchoError::Fallthrough)?;
        }
    }

    if let Ok(result) = submit_result.await {
        result.map_err(|e| match e {
            SubmissionError::IdLocker(..) => anyhow!(t!(already_submitting).clone()).into(),
            e => e,
        })?
    }

    ctx.say(&t!(submitted))
        .await
        .map_err(|_| RikaBanchoError::Fallthrough)?;

    Ok(())
}
