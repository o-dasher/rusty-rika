pub mod en;
pub mod pt_br;

use std::str::FromStr;

use derive_more::From;
use strum::{Display, EnumString};

nestruct::nest! {
    #[derive(Default, Debug, bevy_reflect::Reflect)]
    RikaLocalizer {
        avatar: {
            footer: {
                eq: lexicon::R?,
                other: lexicon::GR<String>?
            }
        },
    }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Display, From, Default, EnumString)]
pub enum RikaLocale {
    #[default]
    #[strum(serialize = "en-US")]
    UnitedStatesEnglish,

    #[strum(serialize = "pt-BR")]
    BrazilianPortuguese,
}

impl From<Option<&str>> for RikaLocale {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(v) => RikaLocale::from_str(v).unwrap_or_default(),
            None => RikaLocale::default(),
        }
    }
}
