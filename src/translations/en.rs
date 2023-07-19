use lexicon::*;

use super::rika_localizer::{
    math::{calc::Calc, Math},
    osu::{link::Link, recommend::Recommend, submit::Submit, Osu},
    rate::Rate,
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
            osu: Osu {
                link: Link {
                    failed: r!(|who| "Failed to link to {who}"),
                    linked: r!(|who| "Linked to osu! account {who}"),
                },
                submit: Submit {
                    submitted: r!("Submitted scores successfullty!"),
                    too_long_warning: r!("This might take a while"),
                    progress_shower: r!(|(amount, out_of)| "Submitted {amount}/{out_of} scores."),
                },
                recommend: Recommend {
                    recommendation: r!(|(link, mods)| {
                        "I recommend you to play {link} with the following mods: {mods}"
                    }),
                    not_found: r!("Could not find any map to recommend for you!"),
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
            rate: Rate {
                rated: r!(|(who, rating)| "{who} is for sure a {rating}..."),
                feedback: Some(
                    [
                        vec!["You should probably just change it already..."],
                        vec!["Meh, not even okay... but if it works for you..."],
                        vec!["Terrible, terribleness. Change it quickly."],
                        vec!["Satoko is prettier than this fr."],
                        vec!["I mean, it is ok... not that bad ig!"],
                        vec!["I mean, if this was a school grade it would be the average ig."],
                        vec!["Wow, that is kinda hot hehe..."],
                        vec!["Can you give me their number, just for something..."],
                        vec!["Wowie, they should open a... you know it!"],
                        vec!["OH GOD! give it to me, gimme gimme (a man of the midnight)"],
                    ]
                    .iter()
                    .map(|v| v.iter().map(|l| l.to_string()).collect())
                    .collect(),
                ),
            },
        }
    }
}
