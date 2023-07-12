use lexicon::{r, wo, DefaultLocalizer, GR};

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
                name: r!("OWO1"),
                avatar: Avatar {
                    name: r!("MIAU"),
                    footer: Footer {
                        eq: r!("Woah, it's you"),
                        other: Some(GR::new(|who| format!("Woah it's {who}"))),
                    },
                },
            },
        }
    }
}
