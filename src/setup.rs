use std::sync::Arc;

use id_locked::IDLocker;
use lexicon::Localizer;
use log::{error, info};
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
    let to_register = &framework.options().commands;

    match config.development_guild {
        Some(dev_guild) => {
            poise::builtins::register_in_guild(ctx, to_register, GuildId(dev_guild)).await?;
        }
        None => poise::builtins::register_globally(ctx, to_register).await?,
    }

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

    let rika_data = Arc::new(RikaData {
        config,
        locales,
        rosu,
        db,
        beatmap_cache: BeatmapCache::new(),
        submit_locker: IDLocker::new(),
    });

    let cloned_data = rika_data.clone();

    tokio::spawn(background_setup(cloned_data));

    Ok(rika_data)
}

async fn background_setup(data: Arc<RikaData>) {
    let RikaData {
        rosu, db, config, ..
    } = data.as_ref();

    let mut scraped_modes = [GameMode::Osu, GameMode::Taiko, GameMode::Mania]
        .into_iter()
        .cycle();

    for page in (1..100).cycle() {
        let Some(mode) = scraped_modes.next() else {
            break;
        };

        let rank = rosu
            .performance_rankings(mode)
            .country(config.scraped_country.clone())
            .page(page)
            .await;

        let Ok(rank) = rank else {
            break;
        };

        for (i, u) in rank.ranking.iter().enumerate() {
            let data = data.clone();

            let id = u.user_id;
            let rosu_user = rosu.user(id).await;

            if let Err(..) = rosu_user {
                break;
            }

            let created_user = OsuUser::try_create(&id).execute(db).await;
            let number_at = 50 * (page as usize - 1) + (i + 1);

            if let Ok(..) = created_user {
                match submit_scores(data, id, mode, None).await {
                    Ok(..) => info!("Submitted scores for top user: {id} at {number_at}"),
                    Err(e) => error!("{e:?}"),
                };
            }
        }
    }
}
