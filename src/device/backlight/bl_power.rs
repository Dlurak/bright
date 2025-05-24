use derive_more::Display;
use std::{io, num::ParseIntError};
use thiserror::Error;

#[repr(u8)]
#[derive(Display)]
pub enum BlPower {
    #[display("on")]
    On = 0,
    #[display("off")]
    Off = 4,
}

impl From<BlPower> for u8 {
    fn from(value: BlPower) -> Self {
        value as Self
    }
}

impl BlPower {
    pub fn try_new(num: u8) -> Option<Self> {
        match num {
            0 => Some(Self::On),
            4 => Some(Self::Off),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum BlPowerReadError {
    #[error("can't parse the content as a number")]
    Parse(
        #[from]
        #[source]
        ParseIntError,
    ),
    #[error("can't read the file")]
    Read(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("the found number {_0} is invalid")]
    InvalidNumber(u8),
}
