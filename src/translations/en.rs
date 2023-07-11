use lexicon::{r, wo, DefaultLocalizer, GR};

use super::rika_localizer::{
    avatar::{footer::Footer, Avatar},
    RikaLocalizer,
};

impl DefaultLocalizer for RikaLocalizer {
    fn default_localizer() -> Self {
        Self {
            avatar: Avatar {
                footer: wo! {
                    Footer
                        eq: r!("Woah, it's you"),
                        other: GR::new(|who| r!("Woah it's {who}")),
                },
            },
        }
    }
}
