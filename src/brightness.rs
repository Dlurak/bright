use crate::{
    animation::easing::Easing,
    device::{Device, errors::DeviceReadError},
};
use derive_more::Display;
use num_traits::Unsigned;
use std::{error, fmt, num::IntErrorKind, path::PathBuf, str::FromStr};
use thiserror::Error;

const EMPTY_ERR_MSG: &str = "Number must not be empty";

pub trait AbsoluteBrightness {
    type Number: Unsigned;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
        easing: &dyn Easing,
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
    Other(Box<dyn error::Error>),
    #[error("the file {} doesn't exist", _0.display())]
    MissingFile(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum Value {
    #[display("{_0}")]
    Absolute(u16),
    #[display("{_0}%")]
    Percentage(f64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        easing: &dyn Easing,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        match self {
            Self::Absolute(a) => Ok(*a),
            Self::Percentage(p) => {
                let max = device.max().ok_or(AbsoluteBrightnessError::NoMax)?;
                let factor = easing.to_actual(p / 100.0);
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
        easing: &dyn Easing,
    ) -> Result<Self::Number, AbsoluteBrightnessError> {
        // Here instead of in the match arm to prevent unneeded reading of the `current` file
        if self.direction == ChangeDirection::Abs {
            return self.value.absolute_brightness(device, easing);
        }

        let current = device
            .current()
            .map_err(AbsoluteBrightnessError::CurrentRead)?;
        let max = device.max().ok_or(AbsoluteBrightnessError::NoMax)?;

        match (self.value, self.direction) {
            (_, ChangeDirection::Abs) => unreachable!(),
            (Value::Absolute(absolute), ChangeDirection::Inc) => {
                Ok(current.saturating_add(absolute).min(max))
            }
            (Value::Absolute(absolute), ChangeDirection::Dec) => {
                Ok(current.saturating_sub(absolute))
            }
            (Value::Percentage(p), direction @ (ChangeDirection::Inc | ChangeDirection::Dec)) => {
                let actual = f64::from(current) / f64::from(max);
                let user_facing = easing.from_actual(actual);
                let current_perc = user_facing * 100.0;
                let perc = match direction {
                    ChangeDirection::Abs => unreachable!(),
                    ChangeDirection::Inc => current_perc + p,
                    ChangeDirection::Dec => current_perc - p,
                };
                Value::Percentage(perc.clamp(0.0, 100.0)).absolute_brightness(device, easing)
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
            .parse()
            .map_err(|_| String::from("Please provide a number"))
            .and_then(|val| {
                if val > 100.0 {
                    Err(String::from("The value must not exceed 100%"))
                } else if val < 0.0 {
                    Err(String::from("The value must be at least 0%"))
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

        s.parse()
            .map_err(|_| String::from("Please provide a number"))
            .and_then(|val| {
                if val > 100.0 {
                    Err(String::from("The value must not exceed 100%"))
                } else if val < 0.0 {
                    Err(String::from("The value must be at least 0%"))
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
                value: Value::Percentage(42.0)
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
