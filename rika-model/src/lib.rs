use std::sync::Arc;

use osu::{beatmap::BeatmapCache, submit::ScoreSubmitter};
use sqlx::MySqlPool;
use tokio::sync::RwLock;

pub mod osu;

pub struct SharedRika {
    pub db: MySqlPool,
    pub rosu: rosu_v2::Osu,
    pub score_submitter: Arc<RwLock<ScoreSubmitter>>,
    pub beatmap_cache: BeatmapCache,
}
