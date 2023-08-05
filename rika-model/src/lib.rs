#![deny(rust_2018_idioms)]

use std::sync::Arc;

use i18n::{rika_localizer::RikaLocalizer, RikaLocale};
use lexicon::Localizer;
use osu::{beatmap::BeatmapCache, submit::ScoreSubmitter};
use sqlx::MySqlPool;
use tokio::sync::RwLock;

pub mod barebone_commands;
pub mod i18n;
pub mod osu;
pub mod rika_cord;

pub struct SharedRika {
    pub db: MySqlPool,
    pub rosu: rosu_v2::Osu,
    pub score_submitter: Arc<RwLock<ScoreSubmitter>>,
    pub beatmap_cache: BeatmapCache,
    pub locales: Localizer<RikaLocale, RikaLocalizer>,
}
