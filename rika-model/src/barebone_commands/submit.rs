use std::sync::Arc;

use anyhow::anyhow;
use async_callable::AsyncCallable1;
use lexicon::{t_prefix, LocaleAccess, Localizer};
use rosu_v2::prelude::GameMode;
use tokio::sync::{
    mpsc,
    oneshot::{self, Sender},
};

use crate::{
    i18n::{rika_localizer::RikaLocalizer, RikaLocale},
    osu::submit::{ScoreSubmitter, SubmissionError, SubmissionID},
    SharedRika,
};

pub enum SubmitStatus<T> {
    Start(Sender<T>),
    Sending((Arc<T>, usize)),
    Finished(Arc<T>),
}

pub async fn submit_barebones<T: Send + Sync + 'static>(
    data: &SharedRika,
    osu_id: impl Into<SubmissionID>,
    i18n: LocaleAccess<'_, Localizer<RikaLocale, RikaLocalizer>>,
    sender: mpsc::UnboundedSender<(SubmitStatus<T>, String)>,
    mode: GameMode,
) -> Result<(), anyhow::Error> {
    t_prefix!($, i18n.osu.submit);

    let SharedRika {
        score_submitter, ..
    } = data;

    let (start_dep_sender, mut start_dep_receiver) = oneshot::channel();

    sender.send((
        SubmitStatus::Start(start_dep_sender),
        t!(too_long_warning).clone(),
    ))?;

    let start_dep = Arc::new(start_dep_receiver.try_recv()?);

    let submission_id: SubmissionID = osu_id.into();

    let (to_submit, mut receiver) = ScoreSubmitter::begin_submission(&score_submitter);
    let submit_result =
        tokio::spawn(async move { to_submit.submit_scores(submission_id, mode).await });

    while let Some((amount, out_of)) = receiver.recv().await {
        sender.send((
            SubmitStatus::Sending((start_dep.clone(), amount)),
            t!(progress_shower).r((amount, out_of)),
        ))?;
    }

    if let Ok(result) = submit_result.await {
        result.map_err(|e| match e {
            SubmissionError::IdLocker(..) => anyhow!(t!(already_submitting).clone()).into(),
            e => e,
        })?
    }

    sender.send((SubmitStatus::Finished(start_dep), t!(submitted).clone()))?;

    Ok(())
}
