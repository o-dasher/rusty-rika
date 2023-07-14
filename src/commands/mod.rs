use crate::RikaError;

pub mod owner;

pub mod user;
pub mod math;
pub mod rate;
pub mod osu;

pub type CommandReturn = Result<(), RikaError>;
