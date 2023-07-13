use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use lexicon::t;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use roricon::RoriconTrait;

use crate::{
    commands::CommandReturn,
    error::RikaError,
    utils::markdown::{bold, mono},
    RikaContext,
};

#[poise::command(slash_command)]
pub async fn waifu(
    ctx: RikaContext<'_>,
    #[description = "The cute waifu you want to rate"] who: String,
) -> CommandReturn {
    let i18n = ctx.i18n();

    let mut hasher = DefaultHasher::new();
    who.hash(&mut hasher);
    let seed_hash = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed_hash);

    let rating = rng.gen_range(1..=10);

    let rated = mono(t!(i18n.rate.rated).r((who, rating.to_string())));
    let feedback = t!(i18n.rate.feedback)
        .get(rating - 1)
        .and_then(|fb| fb.choose(&mut rng))
        .ok_or(RikaError::Fallthrough)?;

    let response = format!("{rated} {feedback}");

    ctx.say(bold(response)).await?;

    Ok(())
}
