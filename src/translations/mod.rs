pub mod en;
pub mod pt_br;

use derive_more::From;
use strum::{Display, EnumString};

nestruct::nest! {
    #[derive(Default, Debug, bevy_reflect::Reflect)]
    RikaLocalizer {
        user: {
            name: lexicon::R?,
            avatar: {
                name: lexicon::R?,
                footer: {
                    eq: lexicon::R?,
                    other: lexicon::GR<String>?
                }
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
