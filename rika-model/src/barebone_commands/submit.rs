use std::sync::Arc;

use anyhow::anyhow;
use derive_more::From;
use lexicon::{t_prefix, LocaleAccess, Localizer};
use rosu_v2::prelude::GameMode;
use tokio::sync::mpsc;

use crate::{
    i18n::{rika_localizer::RikaLocalizer, RikaLocale},
    osu::submit::{ScoreSubmitter, SubmissionError, SubmissionID},
    SharedRika,
};

pub enum SubmitAfter {
    Sending((usize,)),
    Finished,
}

#[derive(From)]
pub enum SubmitStatus {
    Start,
    After(SubmitAfter),
}

pub async fn submit_barebones(
    data: Arc<SharedRika>,
    osu_id: impl Into<SubmissionID>,
    i18n: LocaleAccess<Localizer<RikaLocale, RikaLocalizer>>,
    sender: mpsc::UnboundedSender<(SubmitStatus, String)>,
    mode: GameMode,
) -> Result<(), anyhow::Error> {
    t_prefix!($, i18n.osu.submit);

    let SharedRika {
        score_submitter, ..
    } = data.as_ref();

    sender.send((SubmitStatus::Start, t!(too_long_warning).clone()))?;

    let submission_id: SubmissionID = osu_id.into();

    let (to_submit, mut receiver) = ScoreSubmitter::begin_submission(score_submitter);
    let submit_result =
        tokio::spawn(async move { to_submit.submit_scores(submission_id, mode).await });

    while let Some((amount, out_of)) = receiver.recv().await {
        sender.send((
            SubmitAfter::Sending((amount,)).into(),
            t!(progress_shower).r((amount, out_of)),
        ))?;
        println!("OI");
    }

    if let Ok(result) = submit_result.await {
        result.map_err(|e| match e {
            SubmissionError::IdLocker(..) => anyhow!(t!(already_submitting).clone()).into(),
            e => e,
        })?;
    }

    sender.send((SubmitAfter::Finished.into(), t!(submitted).clone()))?;

    Ok(())
}
