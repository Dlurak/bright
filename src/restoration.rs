use crate::{
    brightness::{AbsoluteBrightness, AbsoluteBrightnessError, BrightnessChange},
    device::UNNAMED,
};
use std::{
    error::Error as StdError,
    fs::{self, File, read_to_string},
    io::{self, ErrorKind, Write},
    path::PathBuf,
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone)]
pub enum SetValue {
    Restore,
    Brightness(BrightnessChange),
}

impl FromStr for SetValue {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "restore" {
            Ok(Self::Restore)
        } else {
            BrightnessChange::from_str(s).map(Self::Brightness)
        }
    }
}

struct Restore;

impl AbsoluteBrightness for Restore {
    type Number = u16;

    fn absolute_brightness(
        &self,
        device: &dyn crate::device::Device<Number = Self::Number>,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        let path = device_restore_path(device.name().unwrap_or(UNNAMED));
        let value = read_to_string(&path).map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                AbsoluteBrightnessError::MissingFile(path)
            } else {
                AbsoluteBrightnessError::Other(Box::new(err) as Box<dyn StdError>)
            }
        })?;
        value
            .parse()
            .map_err(|err| AbsoluteBrightnessError::Other(Box::new(err) as Box<dyn StdError>))
    }
}

impl AbsoluteBrightness for SetValue {
    type Number = u16;
    fn absolute_brightness(
        &self,
        device: &dyn crate::device::Device<Number = Self::Number>,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        match self {
            Self::Restore => Restore.absolute_brightness(device),
            Self::Brightness(brc) => brc.absolute_brightness(device),
        }
    }
}

fn device_restore_path(device_name: &str) -> PathBuf {
    PathBuf::from(format!("/tmp/{}/{device_name}", env!("CARGO_PKG_NAME")))
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("error at directory creation: {_0}")]
    DirCreate(io::Error),
    #[error("error at file creation: {_0}")]
    FileCreate(io::Error),
    #[error("error when writing to file: {_0}")]
    FileWrite(io::Error),
}

pub fn write_brightness(device_name: &str, brightness: u16) -> Result<PathBuf, WriteError> {
    let path = device_restore_path(device_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(WriteError::DirCreate)?;
    }

    let mut file = File::create(&path).map_err(WriteError::FileCreate)?;
    let content = brightness.to_string();
    let content = content.as_bytes();
    match file.write_all(content) {
        Ok(()) => Ok(path),
        Err(err) => Err(WriteError::FileWrite(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            device_restore_path("intel_backlight"),
            PathBuf::from("/tmp/bright/intel_backlight")
        );
    }
}
