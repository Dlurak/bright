use crate::brightness::AbsoluteBrightness;
use derive_more::Display;
use num_traits::Unsigned;
use std::{cmp, collections::HashMap, fmt, path::PathBuf};

pub mod backlight;
pub mod errors;
pub mod led;

pub const BRIGHTNESS_FILES: [&str; 2] = ["brightness", "max_brightness"];

pub trait Device {
    type Number: Unsigned + cmp::Ord + fmt::Display;

    fn name(&self) -> Option<&str>;

    fn max(&self) -> Option<Self::Number>;
    fn current(&self) -> Result<Self::Number, errors::DeviceReadError>;
    fn set(
        &self,
        value: Box<dyn AbsoluteBrightness<Number = Self::Number>>,
    ) -> Result<Self::Number, errors::DeviceWriteError<Self::Number>>;
    fn path(&self) -> Option<PathBuf> {
        None
    }
}

#[derive(PartialEq, Eq, Hash, Display)]
pub enum DeviceClass {
    #[display("Backlight")]
    Backlight,
    #[display("Leds")]
    Leds,
}

impl DeviceClass {
    pub const fn path(&self) -> &str {
        match self {
            Self::Backlight => "/sys/class/backlight/",
            Self::Leds => "/sys/class/leds/",
        }
    }
}

pub fn all_lights() -> HashMap<DeviceClass, Vec<Box<dyn Device<Number = u16>>>> {
    let mut hm = HashMap::new();

    if let Some(backlights) = backlight::find_backlights() {
        let mapped = backlights
            .into_iter()
            .map(|bl| Box::new(bl) as Box<dyn Device<Number = u16>>)
            .collect();

        hm.insert(backlight::Backlight::CLASS, mapped);
    }
    if let Some(leds) = led::find_leds() {
        let mapped = leds
            .into_iter()
            .map(|bl| Box::new(bl) as Box<dyn Device<Number = u16>>)
            .collect();

        hm.insert(led::Led::CLASS, mapped);
    }

    hm
}
