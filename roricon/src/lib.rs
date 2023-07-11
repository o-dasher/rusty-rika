/// This is kind of a disgusting type magic, but i think it is kind of understable
/// given the problem that this is trying to solve.
///
use std::hash::Hash;

use lexicon::{DefaultLocalizer, LocaleAccess, Localizer};

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
        K: Eq + Hash + Copy + Default + From<Option<&'a str>> + 'a,
        V: DefaultLocalizer + 'a,
        U,
        E,
    > RoriconTrait<'a, K, V> for poise::Context<'a, U, E>
where
    Self: RoriconMetaTrait<'a, K, V>,
{
    fn i18n(&self) -> LocaleAccess<K, V> {
        self.locales().get(self.locale())
    }
}

// Reflection nos fields pra achar locales para nomes e descricoes de comandos
