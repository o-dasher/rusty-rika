use std::sync::Arc;

use log::{error, info};
use poise::{
    serenity_prelude::{self, GuildId},
    Framework,
};
use rika_model::{osu::submit::ScoreSubmitter, rika_cord, SharedRika};
use rosu_v2::prelude::GameMode;

use crate::models::osu_user::OsuUser;

pub async fn setup(
    ctx: &serenity_prelude::Context,
    framework: &Framework<Arc<rika_cord::Data>, rika_cord::Error>,
    config: rika_cord::Config,
    shared: Arc<SharedRika>,
) -> Result<Arc<rika_cord::Data>, rika_cord::Error> {
    let to_register = &framework.options().commands;

    match config.development_guild {
        Some(dev_guild) => {
            poise::builtins::register_in_guild(ctx, to_register, GuildId(dev_guild)).await?;
        }
        None => poise::builtins::register_globally(ctx, to_register).await?,
    }

    let rika_data = Arc::new(rika_cord::Data { config, shared });

    let cloned_data = rika_data.clone();

    tokio::spawn(background_setup(cloned_data));

    Ok(rika_data)
}

async fn background_setup(data: Arc<rika_cord::Data>) {
    let rika_cord::Data { config, shared, .. } = data.as_ref();
    let SharedRika {
        db,
        rosu,
        score_submitter,
        ..
    } = shared.as_ref();

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
            let id = u.user_id;
            let rosu_user = rosu.user(id).await;

            if let Err(..) = rosu_user {
                break;
            }

            let created_user = OsuUser::try_create(&id).execute(db).await;
            let number_at = 50 * (page as usize - 1) + (i + 1);

            if let Ok(..) = created_user {
                match ScoreSubmitter::begin_submission(score_submitter)
                    .0
                    .submit_scores(id, mode)
                    .await
                {
                    Ok(..) => info!("Submitted scores for top user: {id} at {number_at}"),
                    Err(e) => error!("{e:?}"),
                };
            }
        }
    }
}
