use crate::RikaError;

pub mod owner;
pub mod user;
pub mod math;

pub type CommandReturn = Result<(), RikaError>;
