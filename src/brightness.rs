use crate::device::Device;
use num_traits::Unsigned;
use std::num::IntErrorKind;

const EMPTY_ERR_MSG: &str = "Number must not be empty";

pub trait AbsoluteBrightness {
    type Number: Unsigned;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Option<Self::Number>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrightnessChange {
    value: Value,
    direction: ChangeDirection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Absolute(u16),
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
    ) -> Option<Self::Number> {
        match self {
            Self::Absolute(a) => Some(*a),
            Self::Percentage(p) => {
                let max = device.max()?;
                let factor = f64::from(*p) / 100.0;
                let f = f64::from(max) * factor;
                Some(f as u16)
            }
        }
    }
}

impl AbsoluteBrightness for BrightnessChange {
    type Number = u16;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Option<Self::Number> {
        let absolute = self.value.absolute_brightness(device)?;
        match self.direction {
            ChangeDirection::Abs => Some(absolute),
            ChangeDirection::Dec => {
                let current = device.current().ok()?;
                Some(current.saturating_sub(absolute))
            }
            ChangeDirection::Inc => {
                let current = device.current().ok()?;
                let max = device.max()?;
                Some(current.saturating_add(absolute).min(max))
            }
        }
    }
}

pub fn brightness_parser(unparsed: &str) -> Result<BrightnessChange, String> {
    let mut unparsed_owned = String::from(unparsed);
    let (mut unparsed, direction) = match unparsed_owned.pop() {
        Some('+') => (unparsed_owned, ChangeDirection::Inc),
        Some('-') => (unparsed_owned, ChangeDirection::Dec),
        Some(_) => (String::from(unparsed), ChangeDirection::Abs),
        None => return Err(String::from(EMPTY_ERR_MSG)),
    };

    match unparsed.parse() {
        Ok(num) => {
            return Ok(BrightnessChange {
                direction,
                value: Value::Absolute(num),
            })
        }
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

    match unparsed.pop() {
        Some('%') => {}
        Some(_) => return Err(String::from("Relative units must end with a '%'")),
        None => unreachable!("We already remove empty ones above"),
    }

    let value = match unparsed.parse::<u8>() {
        Ok(value) => {
            if value > 100 {
                return Err(String::from("The value must not exceed 100%"));
            } else {
                Value::Percentage(value)
            }
        }
        Err(err) => {
            let msg = match err.kind() {
                IntErrorKind::NegOverflow => "The value must be at least 0%",
                IntErrorKind::Empty | IntErrorKind::InvalidDigit => "Please provide a number",
                IntErrorKind::PosOverflow => "The value must not exceed 100%",
                _ => todo!("This is a bug, please create a GitHub issue to report it!"),
            };
            return Err(msg.into());
        }
    };
    Ok(BrightnessChange { value, direction })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(
            brightness_parser("42%+"),
            Ok(BrightnessChange {
                direction: ChangeDirection::Inc,
                value: Value::Percentage(42)
            })
        );
        assert_eq!(
            brightness_parser("42-"),
            Ok(BrightnessChange {
                direction: ChangeDirection::Dec,
                value: Value::Absolute(42)
            })
        );
        assert_eq!(
            brightness_parser("422"),
            Ok(BrightnessChange {
                direction: ChangeDirection::Abs,
                value: Value::Absolute(422)
            })
        );
        assert!(brightness_parser("42++").is_err());
        assert!(brightness_parser("-42").is_err());
        assert!(brightness_parser("-1%").is_err());
        assert!(brightness_parser("101%").is_err());
        assert!(brightness_parser("100%").is_ok());
    }
}
