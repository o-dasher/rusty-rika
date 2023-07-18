use anyhow::anyhow;
use lexicon::t;
use num_traits::Float;
use roricon::RoriconTrait;
use rosu_v2::prelude::GameMods;
use tuple_map::TupleMap4;

use crate::{
    commands::{osu::RikaOsuContext, CommandReturn},
    models::osu_score::OsuPerformance,
    utils::{emojis::RikaMoji, markdown::mono, replies::cool_text},
    RikaContext, RikaData,
};

use super::RikaOsuError;

#[poise::command(slash_command)]
pub async fn recommend(ctx: RikaContext<'_>) -> CommandReturn {
    let i18n = ctx.i18n();
    let RikaData { db, .. } = ctx.data().as_ref();

    let (.., osu_id) = ctx.linked_osu_user().await?;

    let performance_values = sqlx::query_as!(
        OsuPerformance,
        "
        SELECT pp.*
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE osu_user_id = ?
        ",
        osu_id
    )
    .fetch_all(db)
    .await?;

    if performance_values.is_empty() {
        return Err(RikaOsuError::RequiresSubmission.into());
    }

    let weight_to = |f: fn(&OsuPerformance) -> f32| {
        let (pp_sum, weight) = performance_values
            .iter()
            .map(f)
            .enumerate()
            .map(|(i, value)| (value, 0.95f32.powi(i as i32)))
            .map(|(value, weight_by)| (value * weight_by, weight_by))
            .fold((0f32, 0f32), |(pp_sum, weight), (value, weight_by)| {
                (pp_sum + value, weight + weight_by)
            });

        println!("{}", pp_sum / weight);

        pp_sum / weight
    };

    macro_rules! apply_weight {
        ($name:ident, $field:ident) => {
            let $name = weight_to(|v| v.$field);
        };
    }

    apply_weight!(avg_speed, speed);
    apply_weight!(avg_acc, accuracy);
    apply_weight!(avg_aim, aim);
    apply_weight!(avg_fl, flashlight);

    let ((min_speed, max_speed), (min_acc, max_acc), (min_aim, max_aim), (min_fl, max_fl)) =
        (avg_speed, avg_acc, avg_aim, avg_fl).map(|x| {
            fn mid_interval<F: Float>(x: F, delta: F) -> (F, F) {
                let d = delta / F::from(2).unwrap();
                (x * (F::one() - d), x * (F::one() + d))
            }

            mid_interval(x, 0.5)
        });

    let possible_recommendation = sqlx::query!(
        "
        SELECT s.*
        FROM osu_score s
        JOIN osu_performance pp ON s.id = pp.id
        WHERE 
            s.osu_user_id != ? AND
            pp.speed BETWEEN ? AND ? AND
            pp.accuracy BETWEEN ? AND ? AND
            pp.aim BETWEEN ? AND ? AND
            pp.flashlight BETWEEN ? AND ?
        ORDER BY RAND()
        ",
        osu_id,
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
    .map_err(|_| anyhow!(t!(i18n.osu.recommend.not_found).clone()))?;

    let beatmap_link = format!("https://osu.ppy.sh/b/{}", possible_recommendation.map_id);
    let displayable_mods = GameMods::try_from(possible_recommendation.mods)?;

    let content =
        t!(i18n.osu.recommend.recommendation).r((beatmap_link, mono(displayable_mods.to_string())));

    ctx.say(cool_text(RikaMoji::Ok, &content)).await?;

    Ok(())
}
