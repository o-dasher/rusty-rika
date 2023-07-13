use lexicon::*;

use super::rika_localizer::{
    math::{calc::Calc, Math},
    user::{
        avatar::{footer::Footer, Avatar},
        User,
    },
    RikaLocalizer,
};

impl DefaultLocalizer for RikaLocalizer {
    fn default_localizer() -> Self {
        Self {
            math: Math {
                calc: Calc {
                    error_parse_fail: r!(|expr| "Failed to parse {expr}"),
                    results_in: r!(|(expr, res)| "Hai! {expr} results in {res}"),
                },
            },
            user: User {
                avatar: Avatar {
                    footer: Footer {
                        eq: r!("Woah, it's you"),
                        other: r!(|who| "Woah it's {who}"),
                    },
                },
            },
        }
    }
}
