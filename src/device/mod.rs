use derive_more::Display;
use errors::DeviceNotFound;
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
        // value: Box<dyn AbsoluteBrightness<Number = Self::Number>>,
        value: Self::Number,
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

pub fn all_devices() -> HashMap<DeviceClass, Vec<Box<dyn Device<Number = u16>>>> {
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

pub fn get_device<S: AsRef<str>>(
    dev: Option<S>,
) -> Result<Box<dyn Device<Number = u16>>, DeviceNotFound> {
    let devices = all_devices();
    #[allow(clippy::single_match_else)] // this is easier to read as a match
    match dev {
        Some(dev) => {
            let dev = dev.as_ref();
            let backlight = devices.into_values().flatten().find_map(|device| {
                let name = device.name()?;
                (name == dev).then_some(device)
            });
            backlight.ok_or(DeviceNotFound::NoNamed {
                name: dev.to_string(),
            })
        }
        None => {
            let dev = devices
                .into_values()
                .find_map(|list| list.into_iter().next());
            dev.ok_or(DeviceNotFound::NoFound)
        }
    }
}
