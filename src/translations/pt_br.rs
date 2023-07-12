use lexicon::r;

use super::rika_localizer::{RikaLocalizer, user::User};

pub fn locale_pt_br() -> RikaLocalizer {
    RikaLocalizer {
        user: User {
            name: r!("usuario"),
            ..Default::default()
        },
        ..Default::default()
    }
}
