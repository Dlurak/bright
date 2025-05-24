use derive_more::Display;
use std::io;
use thiserror::Error;

#[derive(Display)]
pub enum BlType {
    #[display("raw")]
    Raw,
    #[display("platform")]
    Platform,
    #[display("firmware")]
    Firmware,
}

impl TryFrom<&str> for BlType {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "raw" => Ok(Self::Raw),
            "platform" => Ok(Self::Platform),
            "firmware" => Ok(Self::Firmware),
            _ => Err(()),
        }
    }
}

impl From<BlType> for &str {
    fn from(value: BlType) -> Self {
        match value {
            BlType::Raw => "raw",
            BlType::Platform => "platform",
            BlType::Firmware => "firmware",
        }
    }
}

#[derive(Debug, Error)]
pub enum BlTypeReadError {
    #[error("unsupported content")]
    Parse(String),
    #[error("can't read the file")]
    Read(
        #[from]
        #[source]
        io::Error,
    ),
}
