use crate::device::{Device, errors::DeviceReadError};
use derive_more::Display;
use num_traits::Unsigned;
use std::{error::Error as StdError, fmt, num::IntErrorKind, path::PathBuf, str::FromStr};
use thiserror::Error;

const EMPTY_ERR_MSG: &str = "Number must not be empty";

pub trait AbsoluteBrightness {
    type Number: Unsigned;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Result<Self::Number, AbsoluteBrightnessError>;
}

#[derive(Debug, Error)]
pub enum AbsoluteBrightnessError {
    #[error("maximum brightness for device could not be determined")]
    NoMax,
    // TODO: Are these 2 error attributes really needed
    #[error("{_0}")]
    CurrentRead(DeviceReadError),
    #[error("{_0}")]
    Other(Box<dyn StdError>),
    #[error("the file {} doesn't exist", _0.display())]
    MissingFile(PathBuf),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrightnessChange {
    value: Value,
    direction: ChangeDirection,
}

impl Default for BrightnessChange {
    fn default() -> Self {
        Self {
            value: Value::Absolute(0),
            direction: ChangeDirection::Inc,
        }
    }
}

impl fmt::Display for BrightnessChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let change = match self.direction {
            ChangeDirection::Inc => "+",
            ChangeDirection::Dec => "-",
            ChangeDirection::Abs => "",
        };
        write!(f, "{}{change}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display)]
pub enum Value {
    #[display("{_0}")]
    Absolute(u16),
    #[display("{_0}%")]
    Percentage(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeDirection {
    Dec,
    Abs,
    Inc,
}

impl AbsoluteBrightness for Value {
    type Number = u16;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        match self {
            Self::Absolute(a) => Ok(*a),
            Self::Percentage(p) => {
                let max = device.max().ok_or(AbsoluteBrightnessError::NoMax)?;
                let factor = f64::from(*p) / 100.0;
                let f = f64::from(max) * factor;
                Ok(f as u16)
            }
        }
    }
}

impl AbsoluteBrightness for BrightnessChange {
    type Number = u16;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        let absolute = self.value.absolute_brightness(device)?;
        match self.direction {
            ChangeDirection::Abs => Ok(absolute),
            ChangeDirection::Dec => {
                let current = device
                    .current()
                    .map_err(AbsoluteBrightnessError::CurrentRead)?;
                Ok(current.saturating_sub(absolute))
            }
            ChangeDirection::Inc => {
                let current = device
                    .current()
                    .map_err(AbsoluteBrightnessError::CurrentRead)?;
                let max = device.max().ok_or(AbsoluteBrightnessError::NoMax)?;
                Ok(current.saturating_add(absolute).min(max))
            }
        }
    }
}

impl TryFrom<String> for Value {
    type Error = String;

    fn try_from(mut value: String) -> Result<Self, Self::Error> {
        match value.parse() {
            Ok(num) => return Ok(Self::Absolute(num)),
            Err(err) => match err.kind() {
                IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                    return Err(format!(
                        "Your provided width doesn't fit into the integer (0-{})",
                        u16::MAX
                    ));
                }
                _ => {}
            },
        }

        match value.pop() {
            Some('%') => {}
            Some(_) => return Err(String::from("Relative units must end with a '%'")),
            None => return Err(String::from(EMPTY_ERR_MSG)),
        }

        value
            .parse::<u8>()
            .map_err(|err| {
                let s = match err.kind() {
                    IntErrorKind::NegOverflow => "The value must be at least 0%",
                    IntErrorKind::Empty | IntErrorKind::InvalidDigit => "Please provide a number",
                    IntErrorKind::PosOverflow => "The value must not exceed 100%",
                    _ => unreachable!("This is a bug, please create a GitHub issue to report it!"),
                };
                String::from(s)
            })
            .and_then(|val| {
                if val > 100 {
                    Err(String::from("The value must not exceed 100%"))
                } else {
                    Ok(Self::Percentage(val))
                }
            })
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(num) => return Ok(Self::Absolute(num)),
            Err(err) => match err.kind() {
                IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                    return Err(format!(
                        "Your provided width doesn't fit into the integer (0-{})",
                        u16::MAX
                    ));
                }
                _ => {}
            },
        }

        let mut s = String::from(s);
        match s.pop() {
            Some('%') => {}
            Some(_) => return Err(String::from("Relative units must end with a '%'")),
            None => return Err(String::from(EMPTY_ERR_MSG)),
        }

        s.parse::<u8>()
            .map_err(|err| {
                let s = match err.kind() {
                    IntErrorKind::NegOverflow => "The value must be at least 0%",
                    IntErrorKind::Empty | IntErrorKind::InvalidDigit => "Please provide a number",
                    IntErrorKind::PosOverflow => "The value must not exceed 100%",
                    _ => unreachable!("This is a bug, please create a GitHub issue to report it!"),
                };
                String::from(s)
            })
            .and_then(|val| {
                if val > 100 {
                    Err(String::from("The value must not exceed 100%"))
                } else {
                    Ok(Self::Percentage(val))
                }
            })
    }
}

impl TryFrom<&str> for BrightnessChange {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut value_owned = String::from(value);
        let (value, direction) = match value_owned.pop() {
            Some('+') => (value_owned, ChangeDirection::Inc),
            Some('-') => (value_owned, ChangeDirection::Dec),
            Some(_) => (String::from(value), ChangeDirection::Abs),
            None => return Err(String::from(EMPTY_ERR_MSG)),
        };
        let value = value.try_into()?;

        Ok(Self { value, direction })
    }
}

impl FromStr for BrightnessChange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(
            "42%+".try_into(),
            Ok(BrightnessChange {
                direction: ChangeDirection::Inc,
                value: Value::Percentage(42)
            })
        );
        assert_eq!(
            "42-".try_into(),
            Ok(BrightnessChange {
                direction: ChangeDirection::Dec,
                value: Value::Absolute(42)
            })
        );
        assert_eq!(
            "422".try_into(),
            Ok(BrightnessChange {
                direction: ChangeDirection::Abs,
                value: Value::Absolute(422)
            })
        );
        assert!(BrightnessChange::try_from("42++").is_err());
        assert!(BrightnessChange::try_from("-42").is_err());
        assert!(BrightnessChange::try_from("-1%").is_err());
        assert!(BrightnessChange::try_from("101%").is_err());
        assert!(BrightnessChange::try_from("100%").is_ok());
    }
}
