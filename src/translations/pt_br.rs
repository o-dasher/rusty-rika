use lexicon::{r, wo, GR};

use super::rika_localizer::{
    avatar::{footer::Footer, Avatar},
    RikaLocalizer,
};

pub fn locale_pt_br() -> RikaLocalizer {
    RikaLocalizer {
        avatar: Avatar {
            footer: wo! {
                Footer
                    eq: r!("Woah, é você!"),
                    other: GR::new(|quem| r!("Olha, o {quem}")),
            },
        },
    }
}
