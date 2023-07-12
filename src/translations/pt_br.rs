use lexicon::{r, GR};

use super::rika_localizer::{
    user::{
        avatar::{footer::Footer, Avatar},
        User,
    },
    RikaLocalizer,
};

pub fn locale_pt_br() -> RikaLocalizer {
    RikaLocalizer {
        user: User {
            avatar: Avatar {
                footer: Footer {
                    eq: r!("Eita, é você!"),
                    other: r!(|who| "O avatar do {who} é realmente bem bonito!"),
                },
            },
        },
        ..Default::default()
    }
}
