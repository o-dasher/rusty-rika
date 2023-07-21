pub mod en;
pub mod pt_br;

use derive_more::From;
use strum::{Display, EnumString};

nestruct::nest! {
    #[derive(Default, Debug, bevy_reflect::Reflect)]
    RikaLocalizer {
        math: {
            calc: {
                error_parse_fail: lexicon::GR<String>?,
                results_in: lexicon::GR<(String, String)>?
            }
        },
        osu: {
            link: {
                failed: lexicon::GR<String>?,
                linked: lexicon::GR<String>?
            },
            submit: {
                submitted: lexicon::R?,
                too_long_warning: lexicon::R?,
                progress_shower: lexicon::GR<(usize, usize)>?,
                already_submitting: lexicon::R?
            },
            recommend: {
                recommendation: lexicon::GR<(String, String)>?,
                not_found: lexicon::R?
            }
        },
        user: {
            avatar: {
                footer: {
                    eq: lexicon::R?,
                    other: lexicon::GR<String>?
                }
            }
        },
        rate: {
            rated: lexicon::GR<(String, String)>?,
            feedback: Vec<Vec<String>>?
        }
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
