/// This is kind of a disgusting type magic, but i think it is kind of understable
/// given the problem that this is trying to solve.
///
use std::hash::Hash;

use bevy_reflect::Reflect;
use itertools::{iproduct, Itertools};
use lexicon::{
    DefaultLocalizer, LexiconThroughPath, LocaleAccess, LocaleFromOptionString, Localizer, R,
};
use strum::{Display, EnumIter, IntoEnumIterator};

pub trait RoriconMetaTrait<'a, K: Eq + Hash + Copy + Default + 'a, V: DefaultLocalizer + 'a> {
    // Returns references to the required locales.
    fn locales(&self) -> &'a Localizer<K, V>;
}

/// Automatically implemented trait for context's that provide locales.
pub trait RoriconTrait<K: Eq + Hash + Copy + Default, V: DefaultLocalizer> {
    // Acquires i18n access.
    fn i18n(&self) -> LocaleAccess<K, V>;
}

impl<
        'a,
        K: Eq + Hash + Copy + Default + LocaleFromOptionString + 'a,
        V: DefaultLocalizer + 'a,
        U,
        E,
    > RoriconTrait<K, V> for poise::Context<'a, U, E>
where
    Self: RoriconMetaTrait<'a, K, V>,
{
    fn i18n(&self) -> LocaleAccess<K, V> {
        self.locales().get(K::from_option_locale(self.locale()))
    }
}

#[derive(Display, EnumIter, Clone)]
#[strum(serialize_all = "snake_case")]
enum CommandLocalization {
    Name,
    Description,
}

type LocaleAccesses<'a, K, V> = Vec<(&'a K, LocaleAccess<'a, K, V>)>;

pub fn apply_translations<
    K: Eq + Hash + Copy + Default + ToString,
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
        .map(|key| (key, localizer.get(*key)))
        .collect_vec();

    apply_translation(commands, &locale_accesses)
}

fn apply_translation<
    'a,
    K: Eq + Hash + Copy + Default + ToString,
    V: DefaultLocalizer + Reflect,
    U,
    E,
>(
    commands: &mut [poise::Command<U, E>],
    locale_accesses: &LocaleAccesses<'a, K, V>,
) {
    for command in &mut *commands {
        // Recursive case to apply on subcommands too.
        apply_translation(&mut command.subcommands, &locale_accesses);

        let locale_tags = CommandLocalization::iter()
            .map(|l| (l.clone(), format!("{}.{}", command.name, l)))
            .collect_vec();

        let permutations = iproduct!(locale_accesses, &locale_tags);

        for ((lang_key, access), (locale_type, tag)) in permutations {
            let possible_resource = access.rs::<R>(&tag);

            let Some(localized_key) = possible_resource else {
                continue;
            };

            let applied_locale = lang_key.to_string();
            let localized_key = localized_key.clone();

            match locale_type {
                CommandLocalization::Name => {
                    command
                        .name_localizations
                        .insert(applied_locale, localized_key);
                }
                CommandLocalization::Description => {
                    command
                        .description_localizations
                        .insert(applied_locale, localized_key);
                }
            };
        }
    }
}
