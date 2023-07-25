/// This is kind of a disgusting type magic, but i think it is kind of understable
/// given the problem that this is trying to solve.
///
use std::{collections::HashMap, fmt::Display, hash::Hash, str::FromStr};

use bevy_reflect::Reflect;
use itertools::{iproduct, Itertools};
use lexicon::{
    DefaultLocalizer, LexiconThroughPath, LocaleAccess, LocaleKey, Localizer, LocalizerTrait, R,
};
use poise::{Command, CommandParameter, CommandParameterChoice};
use strum::{Display, EnumIter, IntoEnumIterator};

pub trait RoriconMetaTrait<'a, K: Eq + Hash + Default + Copy, V: DefaultLocalizer> {
    // Returns references to the required locales.
    fn locales(&self) -> &'a Localizer<K, V>;
}

/// Automatically implemented trait for context's that provide locales.
pub trait RoriconTrait<'a, K: Eq + Hash + Default + Copy, V: DefaultLocalizer> {
    // Acquires i18n access.
    fn i18n(&self) -> LocaleAccess<'a, Localizer<K, V>>;
    fn i18n_explicit(&self, localizer: &'a Localizer<K, V>) -> LocaleAccess<'a, Localizer<K, V>>;
}

impl<'a, K: Eq + Hash + Default + Copy + FromStr, V: DefaultLocalizer, U, E> RoriconTrait<'a, K, V>
    for poise::Context<'a, U, E>
where
    Self: RoriconMetaTrait<'a, K, V>,
{
    fn i18n(&self) -> LocaleAccess<'a, Localizer<K, V>> {
        self.i18n_explicit(self.locales())
    }

    fn i18n_explicit(&self, localizer: &'a Localizer<K, V>) -> LocaleAccess<'a, Localizer<K, V>> {
        let key: K = LocaleKey::from(self.locale()).0;
        localizer.get(key)
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
        .map(|key| (key.to_string(), localizer.get(*key)))
        .collect_vec();

    apply_translation(commands, &LocaleAccesses(locale_accesses))
}

trait RoriconLocalizable {
    fn name_localizations(&mut self) -> &mut HashMap<String, String>;
    fn description_localizations(&mut self) -> Option<&mut HashMap<String, String>>;
}

macro_rules! impl_localizable {
    ($struct:ident) => {
        impl<U, E> RoriconLocalizable for $struct<U, E> {
            fn name_localizations(&mut self) -> &mut HashMap<String, String> {
                &mut self.name_localizations
            }

            fn description_localizations(&mut self) -> Option<&mut HashMap<String, String>> {
                Some(&mut self.description_localizations)
            }
        }
    };
}

impl_localizable!(Command);
impl_localizable!(CommandParameter);

impl RoriconLocalizable for CommandParameterChoice {
    fn name_localizations(&mut self) -> &mut HashMap<String, String> {
        &mut self.localizations
    }

    fn description_localizations(&mut self) -> Option<&mut HashMap<String, String>> {
        None
    }
}

fn apply_localization<'a, L: LocalizerTrait>(
    path: &mut Vec<String>,
    next_tag: String,
    localizable: &'a mut impl RoriconLocalizable,
    locale_accesses: &LocaleAccesses<'a, L>,
) where
    L::Key: Display,
    L::Value: Reflect,
{
    path.push(next_tag);

    let locale_tags = CommandLocalization::iter()
        .map(|l| {
            let mut path_new = path.clone();

            path_new.push(l.to_string());

            let path_string = path.iter().join(".");

            (l, path_string)
        })
        .collect_vec();

    // All combinations of locale acesses and locale tags that can
    // be used for this command.
    let permutations = iproduct!(&locale_accesses.0, &locale_tags);

    for ((lang_key, access), (locale_type, tag)) in permutations {
        let possible_resource = access.rs::<R>(tag);

        let Some(localized_key) = possible_resource else {
                continue;
        };

        let lang_key = lang_key.clone();
        let localized_key = localized_key.clone();

        match locale_type {
            CommandLocalization::Name => {
                localizable
                    .name_localizations()
                    .insert(lang_key, localized_key);
            }
            CommandLocalization::Description => {
                match localizable.description_localizations() {
                    Some(v) => v.insert(lang_key, localized_key),
                    None => {
                        continue;
                    }
                };
            }
        };
    }
}

fn apply_translation<L: LocalizerTrait, U, E>(
    commands: &mut [poise::Command<U, E>],
    locale_accesses: &LocaleAccesses<'_, L>,
) where
    L::Key: Display,
    L::Value: Reflect,
{
    for command in commands {
        let mut path_vec = vec![];

        // Recursive case to apply on subcommands too.
        apply_translation(&mut command.subcommands, &locale_accesses);

        // This could be recursive, we could have a trait that defines Children.
        // and we keep calling apply_localization to all the children of the
        // children of the child... Yeah, you get it.
        apply_localization(
            &mut path_vec,
            command.name.clone(),
            command,
            locale_accesses,
        );

        for param in &mut command.parameters {
            apply_localization(&mut path_vec, param.name.clone(), param, locale_accesses);

            for choice in &mut param.choices {
                apply_localization(&mut path_vec, choice.name.clone(), choice, locale_accesses)
            }
        }
    }
}
