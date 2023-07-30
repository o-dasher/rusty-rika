use rika_model::rika_cord;

pub mod owner;

pub mod math;
pub mod osu;
pub mod rate;
pub mod user;

pub type CommandReturn = Result<(), rika_cord::Error>;
