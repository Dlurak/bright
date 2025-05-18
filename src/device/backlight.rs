// Nearly identical to led, add more functionallity to have an actual advantage of having it
// seperately

use super::{
    BRIGHTNESS_FILES, Device, DeviceClass,
    errors::{DeviceReadError, DeviceWriteError},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
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
    path: PathBuf,
    max: u16,
}

impl Device for Backlight {
    type Number = u16;

    fn name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    fn max(&self) -> Option<Self::Number> {
        Some(self.max)
    }

    fn current(&self) -> Result<Self::Number, DeviceReadError> {
        let path = self.path.join("brightness");
        fs::read_to_string(path)?
            .trim_end()
            .parse()
            .map_err(DeviceReadError::from)
    }

    fn set(&self, value: Self::Number) -> Result<Self::Number, DeviceWriteError<Self::Number>> {
        if value > self.max {
            return Err(DeviceWriteError::Overflow {
                max: self.max,
                provided: value,
            });
        }

        let path = self.path.join("brightness");
        let mut file = OpenOptions::new()
            .read(false)
            .create(false)
            .write(true)
            .open(path)?;

        write!(file, "{value}")?;
        Ok(value)
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.path.clone())
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
            let max = fs::read_to_string(path.join("max_brightness"))
                .map_err(DeviceReadError::from)
                .and_then(|max| max.trim_end().parse().map_err(DeviceReadError::from))?;
            Ok(Self { path, max })
        } else {
            Err(NewBacklightError::NotEnoughFiles {
                relevant_files_count,
            })
        }
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
