use std::fmt::Display;

use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum RikaMoji {
    Art,
    X,
}

impl Display for RikaMoji {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_v: &'static str = self.into();
        write!(f, ":{}:", str_v)
    }
}
