use derive_more::Display;
use errors::DeviceNotFound;
use num_traits::Unsigned;
use std::{cmp, collections::BTreeMap, env, fmt, path::PathBuf};

use crate::meta::Meta;

pub mod backlight;
pub mod errors;
pub mod led;

pub const BRIGHTNESS_FILES: [&str; 2] = ["brightness", "max_brightness"];

pub const UNNAMED: &str = "unnamed";

pub trait Device: Meta {
    type Number: Unsigned + cmp::Ord + fmt::Display;

    fn name(&self) -> Option<&str>;

    fn max(&self) -> Option<Self::Number>;
    fn current(&self) -> Result<Self::Number, errors::DeviceReadError>;
    fn set(
        &self,
        value: Self::Number,
    ) -> Result<Self::Number, errors::DeviceWriteError<Self::Number>>;
    fn path(&self) -> Option<PathBuf> {
        None
    }
}

#[derive(PartialEq, Eq, Hash, Display, PartialOrd, Ord)]
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

pub fn all_devices() -> BTreeMap<DeviceClass, Vec<Box<dyn Device<Number = u16>>>> {
    let mut map = BTreeMap::new();

    if let Some(backlights) = backlight::find_backlights() {
        let mapped = backlights
            .into_iter()
            .map(|bl| Box::new(bl) as Box<dyn Device<Number = u16>>)
            .collect();

        map.insert(backlight::Backlight::CLASS, mapped);
    }
    if let Some(leds) = led::find_leds() {
        let mapped = leds
            .into_iter()
            .map(|bl| Box::new(bl) as Box<dyn Device<Number = u16>>)
            .collect();

        map.insert(led::Led::CLASS, mapped);
    }

    map
}

pub fn get_device<S: AsRef<str>>(
    dev: Option<S>,
) -> Result<Box<dyn Device<Number = u16>>, DeviceNotFound> {
    let devices = all_devices();

    let dev = dev
        .map(|d| d.as_ref().to_string())
        .or_else(|| env::var("BRIGHT_DEVICE").ok());

    match dev {
        Some(dev) => {
            let backlight = devices.into_values().flatten().find_map(|device| {
                let name = device.name()?;
                (name == dev).then_some(device)
            });
            backlight.ok_or(DeviceNotFound::NoNamed { name: dev })
        }
        None => {
            let dev = devices
                .into_values()
                .find_map(|list| list.into_iter().next());
            dev.ok_or(DeviceNotFound::NoFound)
        }
    }
}
