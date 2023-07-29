use std::sync::Arc;

use dotenvy::dotenv;
use rika_model::{
    osu::{beatmap::BeatmapCache, submit::ScoreSubmitter},
    SharedRika,
};
use serde::Deserialize;
use sqlx::pool::PoolOptions;
use tokio::{sync::RwLock, try_join};

#[derive(Deserialize)]
pub struct RikaConfig {
    osu_client_id: u64,
    osu_client_secret: String,
    database_url: String,
}

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    let config = envy::from_env::<RikaConfig>().unwrap();

    let rosu = rosu_v2::Osu::builder()
        .client_id(config.osu_client_id)
        .client_secret(&config.osu_client_secret)
        .build()
        .await
        .expect("Failed to connect to osu! api");

    let db = PoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database!");

    let shared_data = Arc::new(SharedRika {
        db,
        rosu,
        beatmap_cache: BeatmapCache::new(),
        score_submitter: Arc::new(RwLock::new(ScoreSubmitter::new())),
    });

    let result_work = try_join!(rika_bancho::run(), rika_poise::run(shared_data.clone()));

    if let Err(e) = result_work {
        println!("{e:?}")
    }
}
