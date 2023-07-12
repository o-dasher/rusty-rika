/// This is kind of a disgusting type magic, but i think it is kind of understable
/// given the problem that this is trying to solve.
///
use std::{fmt::Display, hash::Hash, str::FromStr};

use bevy_reflect::Reflect;
use itertools::{iproduct, Itertools};
use lexicon::{
    DefaultLocalizer, LexiconThroughPath, LocaleAccess, LocaleKey, Localizer, LocalizerTrait, R,
};
use strum::{Display, EnumIter, IntoEnumIterator};

pub trait RoriconMetaTrait<'a, K: Eq + Hash + Default + Copy, V: DefaultLocalizer> {
    // Returns references to the required locales.
    fn locales(&self) -> &'a Localizer<K, V>;
}

/// Automatically implemented trait for context's that provide locales.
pub trait RoriconTrait<'a, K: Eq + Hash + Default + Copy, V: DefaultLocalizer> {
    // Acquires i18n access.
    fn i18n(&self) -> LocaleAccess<'a, Localizer<K, V>>;
}

impl<'a, K: Eq + Hash + Default + Copy + FromStr, V: DefaultLocalizer, U, E> RoriconTrait<'a, K, V>
    for poise::Context<'a, U, E>
where
    Self: RoriconMetaTrait<'a, K, V>,
{
    fn i18n(&self) -> LocaleAccess<'a, Localizer<K, V>> {
        let key: K = LocaleKey::from(self.locale()).0;
        self.locales().get(key)
    }
}

#[derive(Display, EnumIter, Clone)]
#[strum(serialize_all = "snake_case")]
enum CommandLocalization {
    Name,
    Description,
}

struct LocaleAccesses<'a, L: LocalizerTrait>(Vec<(String, LocaleAccess<'a, L>)>);

pub fn apply_translations<
    K: Eq + Hash + Default + Copy + Display,
    V: DefaultLocalizer + Reflect,
    U,
    E,
>(
    commands: &mut [poise::Command<U, E>],
    localizer: &Localizer<K, V>,
) {
    let locale_accesses = localizer
        .store
        .0
        .keys()
        .into_iter()
        .map(|key| (key.to_string(), localizer.get(*key)))
        .collect_vec();

    apply_translation(commands, &LocaleAccesses(locale_accesses))
}

fn apply_translation<'a, L: LocalizerTrait, U, E>(
    commands: &mut [poise::Command<U, E>],
    locale_accesses: &LocaleAccesses<'a, L>,
) where
    L::Key: Display,
    L::Value: Reflect,
{
    for command in &mut *commands {
        // Recursive case to apply on subcommands too.
        apply_translation(&mut command.subcommands, &locale_accesses);

        let locale_tags = CommandLocalization::iter()
            .map(|l| (l.clone(), format!("{}.{}", command.name, l)))
            .collect_vec();

        // All combinations of locale acesses and locale tags that can
        // be used for this command.
        let permutations = iproduct!(&locale_accesses.0, &locale_tags);

        for ((lang_key, access), (locale_type, tag)) in permutations {
            let possible_resource = access.rs::<R>(&tag);

            let Some(localized_key) = possible_resource else {
                continue;
            };

            let lang_key = lang_key.clone();
            let localized_key = localized_key.clone();

            match locale_type {
                CommandLocalization::Name => {
                    command.name_localizations.insert(lang_key, localized_key);
                }
                CommandLocalization::Description => {
                    command
                        .description_localizations
                        .insert(lang_key, localized_key);
                }
            };

        }
    }
}
