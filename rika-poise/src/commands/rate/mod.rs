use paste::paste;
use rika_model::rika_cord;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use lexicon::t_prefix;
use poise::command;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use roricon::RoriconTrait;

use super::CommandReturn;
use crate::utils::markdown::{bold, mono};

const REALLY_CUTE: [&str; 2] = ["dasher", "rika"];

async fn execute_rate(ctx: rika_cord::Context<'_>, who: String) -> CommandReturn {
    let i18n = ctx.i18n();
    t_prefix!($, i18n.rate);

    let safe_who = who.to_lowercase();

    let mut hasher = DefaultHasher::new();
    safe_who.hash(&mut hasher);
    let seed_hash = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed_hash);

    let rating = if REALLY_CUTE.contains(&safe_who.as_str()) {
        100
    } else {
        rng.gen_range(1..=10)
    };

    let rated = t!(rated).r((mono(who), rating.to_string()));
    let feedback_list = t!(feedback);

    let feedback = feedback_list
        .get(rating.max(feedback_list.len()) - 1)
        .and_then(|fb| fb.choose(&mut rng))
        .ok_or(rika_cord::Error::Fallthrough)?;

    let response = format!("{rated} {feedback}");

    ctx.say(bold(response)).await?;

    Ok(())
}

macro_rules! create_rate_command {
    ($type:ident) => {
        paste! {
            #[poise::command(slash_command)]
            pub async fn $type(
                ctx: rika_cord::Context<'_>,
                #[description = "The cute " $type " you want to rate"] who: String
            ) -> CommandReturn {
                execute_rate(ctx, who).await
            }
        }
    };
}

create_rate_command!(waifu);
create_rate_command!(husbando);
create_rate_command!(loli);

#[command(slash_command, subcommands("waifu", "husbando", "loli"))]
pub async fn rate(_ctx: rika_cord::Context<'_>) -> CommandReturn {
    Ok(())
}
