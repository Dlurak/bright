use crate::{
    animation::easing::Easing,
    device::UNNAMED,
    fmt_option,
    meta::{Information, Meta},
};

use super::{
    BRIGHTNESS_FILES, Device, DeviceClass,
    errors::{DeviceReadError, DeviceWriteError},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};
use thiserror::Error;

const CLASS: DeviceClass = DeviceClass::Leds;

pub fn find_leds() -> Option<Vec<Led>> {
    let lights = Path::new(CLASS.path())
        .read_dir()
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| Led::try_new(entry.path()).ok())
        .collect();
    Some(lights)
}

#[derive(Debug)]
pub struct Led {
    pub dev_path: PathBuf,
    pub max: u16,
    /// This (private) field makes it unconstructable outside of this module
    _hidden: PhantomData<()>,
}

impl Device for Led {

    fn name(&self) -> Option<&str> {
        self.dev_path.file_name()?.to_str()
    }

    fn max(&self) -> Option<u16> {
        Some(self.max)
    }

    fn current(&self) -> Result<u16, DeviceReadError> {
        let path = self.dev_path.join("brightness");
        fs::read_to_string(path)?
            .trim_end()
            .parse()
            .map_err(DeviceReadError::from)
    }

    fn set(&self, value: u16) -> Result<u16, DeviceWriteError<u16>> {
        if value > self.max {
            return Err(DeviceWriteError::Overflow {
                max: self.max,
                provided: value,
            });
        }

        let path = self.dev_path.join("brightness");
        let mut file = OpenOptions::new()
            .read(false)
            .create(false)
            .write(true)
            .open(path)?;

        write!(file, "{value}")?;
        Ok(value)
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.dev_path.clone())
    }
}

impl Meta for Led {
    fn meta(&self, easing: &dyn Easing) -> Vec<Information> {
        let cur = self.current().ok();
        let max = self.max;
        let actual = cur.map(|cur| f64::from(cur) / f64::from(max));
        let user_facing = actual.map(|ac| easing.from_actual(ac));
        let perc = user_facing.map(|x| x * 100.0);

        vec![
            Information::new(
                "Device".to_string(),
                self.name().unwrap_or(UNNAMED).to_string(),
                Some(self.dev_path.display().to_string()),
            ),
            Information::new(
                "Current brightness".to_string(),
                fmt_option(cur, '?'),
                perc.map(|p| format!("{p}%")),
            ),
            Information::new("Max brightness".to_string(), max.to_string(), None),
        ]
    }
}

impl Led {
    pub const CLASS: DeviceClass = CLASS;

    pub fn try_new(path: PathBuf) -> Result<Self, NewLedError> {
        let entries = path.read_dir()?;
        let relevant_files_count = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let ft = entry.file_type().ok()?;
                ft.is_file().then(|| entry.file_name().into_string().ok())?
            })
            .filter(|name| BRIGHTNESS_FILES.contains(&name.as_str()))
            .take(BRIGHTNESS_FILES.len())
            .count();

        if relevant_files_count == BRIGHTNESS_FILES.len() {
            let content =
                fs::read_to_string(path.join("max_brightness")).map_err(DeviceReadError::from)?;
            let max = content.trim_end().parse().map_err(DeviceReadError::from)?;
            Ok(Self {
                dev_path: path,
                max,
                _hidden: PhantomData,
            })
        } else {
            Err(NewLedError::NotEnoughFiles {
                relevant_files_count,
            })
        }
    }

    /// # Safety
    /// Missing check that all required files for a LED-Device are present
    pub unsafe fn new_unchecked(path: PathBuf) -> Result<Self, DeviceReadError> {
        let content = fs::read_to_string(path.join("max_brightness"))?;
        let max = content.trim_end().parse()?;
        Ok(Self {
            dev_path: path,
            max,
            _hidden: PhantomData,
        })
    }
}

#[derive(Debug, Error)]
pub enum NewLedError {
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
    #[error("only {relevant_files_count} of {} required files were found", BRIGHTNESS_FILES.len())]
    NotEnoughFiles { relevant_files_count: usize },
}
