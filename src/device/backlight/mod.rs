mod bl_power;
mod bl_type;

use super::{
    BRIGHTNESS_FILES, Device, DeviceClass,
    errors::{DeviceReadError, DeviceWriteError},
    led::Led,
};
use crate::{
    config::Easings,
    meta::{Information, Meta},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    num::ParseIntError,
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

const CLASS: DeviceClass = DeviceClass::Backlight;

const BACKLIGHT_FILES: [&str; 3] = ["actual_brightness", "bl_power", "type"];
const ALL_FILES: [&str; BRIGHTNESS_FILES.len() + BACKLIGHT_FILES.len()] = {
    let mut res = [""; 5];
    let mut i = 0;
    while i < BRIGHTNESS_FILES.len() {
        res[i] = BRIGHTNESS_FILES[i];
        i += 1;
    }

    let mut i = 0;
    while i < BACKLIGHT_FILES.len() {
        res[BRIGHTNESS_FILES.len() + i] = BACKLIGHT_FILES[i];
        i += 1;
    }

    res
};

pub fn find_backlights() -> Option<Vec<Backlight>> {
    let lights = Path::new(CLASS.path())
        .read_dir()
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| Backlight::try_new(entry.path()).ok())
        .collect();
    Some(lights)
}

#[derive(Debug)]
pub struct Backlight {
    led: Led,
}

impl Device for Backlight {
    delegate::delegate! {
        to self.led {
            fn name(&self) -> Option<&str>;
            fn max(&self) -> u16;
            fn set(&self, value: u16) -> Result<u16, DeviceWriteError<u16>>;
            fn path(&self) -> Option<PathBuf>;
        }
    }

    fn current(&self) -> Result<u16, DeviceReadError> {
        self.parse("actual_brightness")
    }
}

impl Meta for Backlight {
    fn meta(&self, easings: &Easings) -> Vec<Information> {
        let mut list = self.led.meta(easings);
        if let Ok(mode) = self.power_mode() {
            list.push(Information::new(
                "Power mode".to_string(),
                mode.to_string(),
                None,
            ));
        }

        if let Ok(bl_t) = self.bl_type() {
            list.push(Information::new("Type".to_string(), bl_t.to_string(), None));
        }

        list
    }
}

impl Backlight {
    pub const CLASS: DeviceClass = CLASS;

    pub fn try_new(path: PathBuf) -> Result<Self, NewBacklightError> {
        let entries = path.read_dir()?;
        let relevant_files_count = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let ft = entry.file_type().ok()?;
                ft.is_file().then(|| entry.file_name().into_string().ok())?
            })
            .filter(|name| ALL_FILES.contains(&name.as_str()))
            .take(ALL_FILES.len())
            .count();

        if relevant_files_count == ALL_FILES.len() {
            Ok(Self {
                led: unsafe { Led::new_unchecked(path)? },
            })
        } else {
            Err(NewBacklightError::NotEnoughFiles {
                relevant_files_count,
            })
        }
    }

    pub fn power_mode(&self) -> Result<bl_power::BlPower, bl_power::BlPowerReadError> {
        let path = self.led.dev_path.join("bl_power");
        let num = fs::read_to_string(path)?.trim_end().parse()?;
        bl_power::BlPower::try_new(num).ok_or(bl_power::BlPowerReadError::InvalidNumber(num))
    }

    pub fn set_power(&self, power: bl_power::BlPower) -> io::Result<()> {
        let path = self.led.dev_path.join("bl_power");
        let mut file = OpenOptions::new()
            .read(false)
            .create(false)
            .write(true)
            .open(path)?;

        write!(file, "{}", u8::from(power))
    }

    pub fn bl_type(&self) -> Result<bl_type::BlType, bl_type::BlTypeReadError> {
        let path = self.led.dev_path.join("type");
        let content = fs::read_to_string(path)?;
        bl_type::BlType::try_from(content.trim_end())
            .map_err(|_| bl_type::BlTypeReadError::Parse(content.to_string()))
    }

    fn parse<P, T>(&self, file: P) -> Result<T, DeviceReadError>
    where
        P: AsRef<Path>,
        T: FromStr<Err = ParseIntError>,
    {
        let path = self.led.dev_path.join(file);
        fs::read_to_string(path)?
            .trim_end()
            .parse()
            .map_err(DeviceReadError::from)
    }

    pub fn wanted_brightness(&self) -> Result<u16, DeviceReadError> {
        self.parse("brightness")
    }
}

#[derive(Debug, Error)]
pub enum NewBacklightError {
    #[error("can't list content of directory")]
    DirList(
        #[source]
        #[from]
        io::Error,
    ),
    #[error("can't read maximal brightness")]
    ReadMax(
        #[source]
        #[from]
        DeviceReadError,
    ),
    #[error("only {relevant_files_count} of {} required files were found", ALL_FILES.len())]
    NotEnoughFiles { relevant_files_count: usize },
}
