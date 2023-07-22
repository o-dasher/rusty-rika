use crate::RikaContext;
use crate::{commands::CommandReturn, models::osu_score::OsuScore};
use num_traits::Float;
use poise::command;

mod mania;
mod osu;
mod taiko;

use mania::mania;
use osu::osu;
use sqlx::{MySql, Pool, QueryBuilder};
use taiko::taiko;

#[command(slash_command, subcommands("osu", "taiko", "mania"))]
pub async fn recommend(_ctx: RikaContext<'_>) -> CommandReturn {
    Ok(())
}

pub fn get_weighter<T>(vec: Vec<T>) -> impl Fn(for<'a> fn(&'a T) -> f32) -> f32 {
    move |f: fn(&T) -> f32| {
        let (pp_sum, weight) = vec
            .iter()
            .map(f)
            .enumerate()
            .map(|(i, value)| (value, 0.95f32.powi(i as i32)))
            .map(|(value, weight_by)| (value * weight_by, weight_by))
            .fold((0f32, 0f32), |(pp_sum, weight), (value, weight_by)| {
                (pp_sum + value, weight + weight_by)
            });

        pp_sum / weight
    }
}

pub fn mid_interval<F: Float>(x: F, delta: F) -> (F, F) {
    let d = delta / F::from(2).unwrap();
    (x * (F::one() - d), x * (F::one() + d))
}

#[macro_export]
macro_rules! create_weighter {
    ($performance_values:expr, $range:expr) => {
        let weight_to = get_weighter($performance_values);

        macro_rules! apply_weight {
            ($field:ident) => {{
                mid_interval(weight_to(|v| v.$field), $range)
            }};
        }
    };
}

#[macro_export]
macro_rules! fetch_performance {
    ($mode:ident, $osu_id:expr, $db:expr) => {{
        paste! {
            let row: Vec<[<$mode Performance>]> = sqlx::query_as(&format!(
                "
                SELECT pp.* FROM osu_score s
                JOIN {}_performance pp ON s.id = pp.id WHERE osu_user_id = ?
                ORDER BY pp.overall DESC
                ",
                OsuMode::$mode.to_string().to_lowercase()
            ))
            .bind($osu_id)
            .fetch_all($db)
            .await?;

            if row.is_empty() {
                return Err(RikaOsuError::RequiresSubmission)?;
            }

            row
        }
    }};
}

#[macro_export]
macro_rules! reply_recommendation {
    ($ctx:expr, $recommendation:expr) => {
        let recommendation = $recommendation
            .await
            .map_err(|_| anyhow!(t!(not_found).clone()))?;

        let beatmap_link = format!("https://osu.ppy.sh/b/{}", recommendation.map_id);
        let displayable_mods = GameMods::try_from(recommendation.mods)?;

        let content = t!(recommendation).r((beatmap_link, mono(displayable_mods.to_string())));

        $ctx.say(cool_text(RikaMoji::Ok, &content)).await?;
    };
}

#[macro_export]
macro_rules! init_recommendation {
    ($dollar:tt, $ctx:expr, $range:expr, $mode:ident) => {
        let i18n = $ctx.i18n();
        t_prefix!($dollar, i18n.osu.recommend);

        let RikaData { db, .. } = $ctx.data().as_ref();

        let range = $range.unwrap_or(0.3);
        let (.., osu_id) = $ctx.linked_osu_user().await?;

        create_weighter!(fetch_performance!($mode, osu_id, db), range);
    };
}

async fn query_recommendation<'a>(
    pool: &Pool<MySql>,
    mode: &'a str,
    values: Vec<(&'a str, (f32, f32))>,
) -> Result<OsuScore, sqlx::Error> {
    let mut query = QueryBuilder::<MySql>::new(format!(
        "
        SELECT s.*
        FROM osu_score s
        JOIN {mode}_performance pp ON s.id = pp.id
        WHERE
        "
    ));

    let mut separated = query.separated(" AND ");

    for (name, (min, max)) in values {
        separated.push(format!("pp.{name} BETWEEN "));
        separated.push_bind_unseparated(min);
        separated.push_bind(max);
    }

    query.push(" ORDER BY RAND() ");

    query.build_query_as().fetch_one(pool).await
}
