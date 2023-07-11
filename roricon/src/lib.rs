/// This is kind of a disgusting type magic, but i think it is kind of understable
/// given the problem that this is trying to solve.
///
use std::hash::Hash;

use bevy_reflect::Reflect;
use itertools::Itertools;
use lexicon::{
    DefaultLocalizer, LexiconThroughPath, LocaleAccess, LocaleFromOptionString, Localizer, R,
};
use strum::{Display, EnumIter, IntoEnumIterator};

pub trait RoriconMetaTrait<'a, K: Eq + Hash + Copy + Default + 'a, V: DefaultLocalizer + 'a> {
    // Returns references to the required locales.
    fn locales(&self) -> &'a Localizer<K, V>;
}

/// Automatically implemented trait for context's that provide locales.
pub trait RoriconTrait<'a, K: Eq + Hash + Copy + Default + 'a, V: DefaultLocalizer + 'a> {
    // Acquires i18n access.
    fn i18n(&self) -> LocaleAccess<K, V>;
}

impl<
        'a,
        K: Eq + Hash + Copy + Default + LocaleFromOptionString + 'a,
        V: DefaultLocalizer + 'a,
        U,
        E,
    > RoriconTrait<'a, K, V> for poise::Context<'a, U, E>
where
    Self: RoriconMetaTrait<'a, K, V>,
{
    fn i18n(&self) -> LocaleAccess<K, V> {
        self.locales().get(K::from_option_locale(self.locale()))
    }
}

pub fn apply_translations<
    'a,
    K: Eq + Hash + Copy + Default + ToString,
    V: DefaultLocalizer + Reflect,
    U,
    E,
>(
    commands: &mut [poise::Command<U, E>],
    localizer: &Localizer<K, V>,
) {
    #[derive(Display, EnumIter, Clone)]
    #[strum(serialize_all = "snake_case")]
    enum CommandLocalization {
        Name,
        Description,
    }

    for command in &mut *commands {
        let localization_keys = CommandLocalization::iter()
            .map(|l| (l.clone(), format!("{}.{}", command.name, l)))
            .collect_vec();

        for locale in localizer.store.0.keys().into_iter() {
            let locale_access = localizer.get(*locale);

            for (locale_type, key) in &localization_keys {
                let possible_resource = locale_access.rs::<R>(&key);

                let Some(localized_key) = possible_resource else {
                    continue;
                };

                let localized_key = localized_key.to_string();
                let applied_locale = locale.to_string();

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
}
