use lexicon::*;

use super::rika_localizer::{
    user::{
        avatar::{footer::Footer, Avatar},
        User,
    },
    RikaLocalizer,
};

impl DefaultLocalizer for RikaLocalizer {
    fn default_localizer() -> Self {
        Self {
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
