use crate::RikaError;

pub mod owner;
pub mod user;

pub type CommandReturn = Result<(), RikaError>;
