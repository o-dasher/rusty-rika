use std::sync::Arc;

use lexicon::Localizer;
use log::{error, info, warn};
use poise::{
    serenity_prelude::{self, GuildId},
    Framework,
};
use rosu_v2::prelude::GameMode;
use sqlx::pool::PoolOptions;

use crate::{
    error::RikaError,
    models::osu_user::OsuUser,
    tasks::osu::submit::submit_scores,
    translations::{rika_localizer::RikaLocalizer, RikaLocale},
    utils::osu::BeatmapCache,
    RikaConfig, RikaData,
};

pub async fn setup(
    ctx: &serenity_prelude::Context,
    framework: &Framework<Arc<RikaData>, RikaError>,
    locales: Localizer<RikaLocale, RikaLocalizer>,
    config: RikaConfig,
) -> Result<Arc<RikaData>, RikaError> {
    poise::builtins::register_in_guild(
        ctx,
        &framework.options().commands,
        GuildId(config.development_guild),
    )
    .await?;

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

    let beatmap_cache = BeatmapCache::new();

    let rika_data = Arc::new(RikaData {
        config,
        locales,
        rosu,
        beatmap_cache,
        db,
    });

    let cloned_data = rika_data.clone();

    tokio::spawn(async move {
        let RikaData { rosu, db, .. } = cloned_data.as_ref();

        for page in 1..10 {
            let rank = rosu
                .performance_rankings(GameMode::Osu)
                .country("BR")
                .page(page)
                .await;

            let Ok(rank) = rank else {
                warn!("Stopped processing ranks!");
                break;
            };

            for (i, u) in rank.ranking.iter().enumerate() {
                let id = u.user_id;
                let rosu_user = rosu.user(id).await;

                if let Err(..) = rosu_user {
                    break;
                }

                let created_user = OsuUser::try_create(&id).execute(db).await;
                let number_at = 50 * (page as usize - 1) + (i + 1);

                if let Ok(..) = created_user {
                    match submit_scores(&cloned_data, id).await {
                        Ok(..) => info!("Submitted scores for top user: {id} at {number_at}"),
                        Err(e) => error!("{e:?}"),
                    };
                }
            }
        }
    });

    Ok(rika_data)
}

