use std::{io, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceReadError {
    Parse(
        #[from]
        #[source]
        ParseIntError,
    ),
    Read(
        #[from]
        #[source]
        io::Error,
    ),
}

impl std::fmt::Display for DeviceReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::error::Error;

        let verb = match self {
            Self::Read(_) => "read",
            Self::Parse(_) => "parse",
        };
        match self.source() {
            Some(source) => write!(f, "can't {verb} the brightness ({source})"),
            None => write!(f, "can't {verb} the brightness"),
        }
    }
}

#[derive(Debug, Error)]
pub enum DeviceWriteError<T>
where
    T: std::cmp::Ord + std::fmt::Display,
{
    #[error("provided brightness {provided} is bigger than {max}")]
    Overflow { max: T, provided: T },
    #[error("writing the file failed {}", .0.kind())]
    Write(
        #[from]
        #[source]
        io::Error,
    ),
}

#[derive(Debug, Error)]
pub enum DeviceNotFound {
    #[error("no device named '{name}' available")]
    NoNamed { name: String },
    #[error("no device available")]
    NoFound,
}
