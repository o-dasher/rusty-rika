use anyhow::anyhow;
use itertools::Itertools;
use lexicon::t;
use num_traits::Float;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMods;

use crate::{
    commands::{
        osu::{OsuMode, RikaOsuContext},
        CommandReturn,
    },
    error::RikaError,
    models::osu_score::{ManiaPerformance, OsuPerformance, OsuScore, TaikoPerformance},
    utils::{emojis::RikaMoji, markdown::mono, replies::cool_text},
    RikaContext, RikaData,
};
use derive_more::From;

use super::RikaOsuError;

#[poise::command(slash_command)]
pub async fn recommend(ctx: RikaContext<'_>, mode: OsuMode) -> CommandReturn {
    let i18n = ctx.i18n();
    let RikaData { db, .. } = ctx.data().as_ref();

    let (.., osu_id) = ctx.linked_osu_user().await?;

    #[derive(From)]
    enum PerformanceKind {
        Osu(OsuPerformance),
        Taiko(TaikoPerformance),
        Mania(ManiaPerformance),
    }

    macro_rules! fetch_performance {
        ($type:ty, $query:literal) => {
            sqlx::query_as!($type, $query, osu_id)
                .fetch_all(db)
                .await?
                .into_iter()
                .map_into()
                .collect_vec()
                .into()
        };
    }

    let performance_values: Option<Vec<PerformanceKind>> = match mode {
        OsuMode::Standard => fetch_performance!(
            OsuPerformance,
            "
            SELECT pp.* FROM osu_score s
            JOIN osu_performance pp ON s.id = pp.id WHERE osu_user_id = ?
            "
        ),
        OsuMode::Taiko => fetch_performance!(
            TaikoPerformance,
            "
            SELECT pp.* FROM osu_score s
            JOIN taiko_performance pp ON s.id = pp.id WHERE osu_user_id = ?
            "
        ),
        OsuMode::Mania => fetch_performance!(
            ManiaPerformance,
            "
            SELECT pp.* FROM osu_score s
            JOIN mania_performance pp ON s.id = pp.id WHERE osu_user_id = ?
            "
        ),
        _ => None,
    };

    let Some(performance_values) = performance_values else {
        return Err(RikaOsuError::UnsupportedMode.into())
    };

    if performance_values.is_empty() {
        return Err(RikaOsuError::RequiresSubmission.into());
    }

    fn get_weighter<T>(vec: Vec<T>) -> impl Fn(for<'a> fn(&'a T) -> f32) -> f32 {
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

    fn mid_interval<F: Float>(x: F, delta: F) -> (F, F) {
        let d = delta / F::from(2).unwrap();
        (x * (F::one() - d), x * (F::one() + d))
    }

    macro_rules! create_weighter {
        ($variant:ident) => {
            let weight_to = get_weighter(
                performance_values
                    .into_iter()
                    .filter_map(|k| match k {
                        PerformanceKind::$variant(v) => Some(v),
                        _ => None,
                    })
                    .collect_vec(),
            );

            macro_rules! apply_weight {
                ($field:ident) => {{
                    mid_interval(weight_to(|v| v.$field), 0.3)
                }};
            }
        };
    }

    let possible_recommendation = match mode {
        OsuMode::Standard => {
            create_weighter!(Osu);

            let (min_speed, max_speed) = apply_weight!(speed);
            let (min_acc, max_acc) = apply_weight!(accuracy);
            let (min_aim, max_aim) = apply_weight!(aim);
            let (min_fl, max_fl) = apply_weight!(flashlight);

            sqlx::query_as!(
                OsuScore,
                "
                SELECT s.*
                FROM osu_score s
                JOIN osu_performance pp ON s.id = pp.id
                WHERE 
                    pp.speed BETWEEN ? AND ? AND
                    pp.accuracy BETWEEN ? AND ? AND
                    pp.aim BETWEEN ? AND ? AND
                    pp.flashlight BETWEEN ? AND ?
                ORDER BY RAND()
                ",
                min_speed,
                max_speed,
                min_acc,
                max_acc,
                min_aim,
                max_aim,
                min_fl,
                max_fl
            )
            .fetch_one(db)
            .await
        }
        OsuMode::Taiko => {
            create_weighter!(Taiko);

            let (min_acc, max_acc) = apply_weight!(accuracy);
            let (min_diff, max_diff) = apply_weight!(difficulty);

            sqlx::query_as!(
                OsuScore,
                "
                SELECT s.*
                FROM osu_score s
                JOIN taiko_performance pp ON s.id = pp.id
                WHERE 
                    pp.accuracy BETWEEN ? AND ? AND
                    pp.difficulty BETWEEN ? AND ?
                ORDER BY RAND()
                ",
                min_acc,
                max_acc,
                min_diff,
                max_diff
            )
            .fetch_one(db)
            .await
        }
        OsuMode::Mania => {
            create_weighter!(Mania);

            let (min_diff, max_diff) = apply_weight!(difficulty);

            sqlx::query_as!(
                OsuScore,
                "
                SELECT s.*
                FROM osu_score s
                JOIN mania_performance pp ON s.id = pp.id
                WHERE 
                    pp.difficulty BETWEEN ? AND ?
                ORDER BY RAND()
                ",
                min_diff,
                max_diff
            )
            .fetch_one(db)
            .await
        }
        _ => Err(RikaError::Fallthrough)?,
    }
    .map_err(|_| anyhow!(t!(i18n.osu.recommend.not_found).clone()))?;

    let beatmap_link = format!("https://osu.ppy.sh/b/{}", possible_recommendation.map_id);
    let displayable_mods = GameMods::try_from(possible_recommendation.mods)?;

    let content =
        t!(i18n.osu.recommend.recommendation).r((beatmap_link, mono(displayable_mods.to_string())));

    ctx.say(cool_text(RikaMoji::Ok, &content)).await?;

    Ok(())
}
