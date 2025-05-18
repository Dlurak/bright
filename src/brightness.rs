use crate::device::Device;
use num_traits::Unsigned;
use std::num::IntErrorKind;

#[derive(Clone, Debug)]
pub enum BrightnessValue {
    Absolute(u16),
    Percentage(u8),
}

pub trait AbsoluteBrightness {
    type Number: Unsigned;
    fn absolute_brightness(
        &self,
        device: &dyn Device<Number = Self::Number>,
    ) -> Option<Self::Number>;
}

impl AbsoluteBrightness for BrightnessValue {
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

pub fn brightness_parser(unparsed: &str) -> Result<BrightnessValue, String> {
    match unparsed.parse() {
        Ok(num) => return Ok(BrightnessValue::Absolute(num)),
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

    let mut s = String::from(unparsed);
    match s.pop() {
        Some('%') => {}
        Some(_) => return Err(String::from("Relative units must end with a '%'")),
        None => unreachable!("We already remove empty ones above"),
    }

    match s.parse::<u8>() {
        Ok(value) => {
            if value > 100 {
                Err(String::from("The value must not exceed 100%"))
            } else {
                Ok(BrightnessValue::Percentage(value))
            }
        }
        Err(err) => {
            let msg = match err.kind() {
                IntErrorKind::NegOverflow => "The value must be at least 0%",
                IntErrorKind::Empty | IntErrorKind::InvalidDigit => "Please provide a number",
                IntErrorKind::PosOverflow => "The value must not exceed 100%",
                _ => todo!("This is a bug, please create a GitHub issue to report it!"),
            };
            Err(msg.into())
        }
    }
}
