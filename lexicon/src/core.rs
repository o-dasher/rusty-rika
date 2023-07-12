use std::{collections::HashMap, hash::Hash, str::FromStr};

use bevy_reflect::{FromReflect, GetPath, Reflect, TypePath};

pub type R = String;

fn default_gr<A>() -> fn(A) -> String {
    |_args: A| "".to_string()
}

#[derive(Reflect, Debug)]
pub struct GR<A> {
    #[reflect(ignore)]
    #[reflect(default = "default_gr")]
    pub caller: fn(A) -> String,
}

impl<A> GR<A> {
    pub fn new(caller: fn(A) -> String) -> Self {
        Self { caller }
    }

    pub fn r(&self, args: A) -> String {
        (self.caller)(args)
    }
}

/// A trait for a type that can be localized.
pub trait DefaultLocalizer {
    fn default_localizer() -> Self;
}

/// Stores localizers for a given locale.
pub struct LocalizerStore<L: LocalizerTrait>(pub HashMap<L::Key, L::Value>);

impl<L: LocalizerTrait, F: Fn() -> L::Value> From<Vec<(L::Key, F)>> for LocalizerStore<L> {
    fn from(value: Vec<(L::Key, F)>) -> Self {
        Self(value.into_iter().map(|(k, v)| (k, v())).collect())
    }
}

/// A localizer that wraps a store of localizer implementations.
pub struct Localizer<K: Eq + Hash + Default + Copy, V: DefaultLocalizer> {
    pub store: LocalizerStore<Self>,
}

pub trait LocalizerTrait {
    type Key: Eq + Hash + Copy + Default;
    type Value: DefaultLocalizer;
}

impl<K: Eq + Hash + Copy + Default, V: DefaultLocalizer> LocalizerTrait for Localizer<K, V> {
    type Key = K;
    type Value = V;
}

/// A wrapper for a localizer that provides access to the localizer for a given locale.
pub struct LocaleAccess<'a, L: LocalizerTrait> {
    pub localizer: &'a Localizer<L::Key, L::Value>,
    pub to: &'a L::Value,
}

impl<'a, L: LocalizerTrait> LocaleAccess<'a, L> {
    /// Returns the localized value for the given locale, or the default value if the locale is not
    /// properly declared and found.
    pub fn r<Resource>(&self, accessing: fn(&'a L::Value) -> &'a Option<Resource>) -> &'a Resource {
        accessing(self.to)
            .as_ref()
            .unwrap_or_else(|| accessing(self.localizer.ref_default()).as_ref().unwrap())
    }
}

pub trait LexiconThroughPath {
    // Acquires a resource through a given path, like in ".this.is.pretty.fucked.up.owo";
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, acessing: &str) -> Option<&Resource>;
}

impl<V: DefaultLocalizer + Reflect> LexiconThroughPath for V {
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, acessing: &str) -> Option<&Resource> {
        self.path::<Option<Resource>>(acessing)
            .ok()
            .and_then(|x| x.as_ref())
    }
}

impl<'a, L: LocalizerTrait> LexiconThroughPath for LocaleAccess<'a, L>
where
    L::Value: Reflect,
{
    fn rs<Resource: Reflect + TypePath + FromReflect>(
        &self,
        acessing: &str,
    ) -> Option<&'a Resource> {
        self.to
            .rs(acessing)
            .or_else(|| self.localizer.ref_default().rs(acessing))
    }
}

// A NewType wrapper for a locale key with extended capabilities.
pub struct LocaleKey<K: Eq + Hash + Copy + Default + FromStr>(pub K);

impl<K: Eq + Hash + Copy + Default + FromStr> From<&str> for LocaleKey<K> {
    fn from(value: &str) -> Self {
        LocaleKey(K::from_str(value).unwrap_or_default())
    }
}

impl<K: Eq + Hash + Copy + Default + FromStr> From<Option<&str>> for LocaleKey<K> {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(v) => v.into(),
            None => LocaleKey(K::default()),
        }
    }
}

impl<K: Eq + Hash + Copy + Default, V: DefaultLocalizer> Localizer<K, V>
where
    Self: LocalizerTrait<Key = K, Value = V>,
{
    /// Creates a new localizer from a store of localizers.
    pub fn new(store: Vec<(K, fn() -> V)>) -> Self {
        let mut store = LocalizerStore::from(store);

        store.0.insert(K::default(), V::default_localizer());

        Self { store }
    }

    fn ref_opt(&self, locale: &K) -> Option<&V> {
        self.store.0.get(locale)
    }

    fn ref_default(&self) -> &V {
        self.ref_opt(&K::default()).unwrap()
    }

    fn ref_any(&self, locale: &K) -> &V {
        self.ref_opt(locale).unwrap_or_else(|| self.ref_default())
    }

    /// Returns a wrapper for the localizer that provides access to the localizer for a given locale.
     pub fn get<'a>(&'a self, locale: impl Into<K>) -> LocaleAccess<'a, Self> {
        LocaleAccess {
            localizer: &self,
            to: self.ref_any(&locale.into()),
        }
    }
}
