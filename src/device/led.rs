use super::{
    BRIGHTNESS_FILES, Device, DeviceClass,
    errors::{DeviceReadError, DeviceWriteError},
};
use crate::brightness::AbsoluteBrightness;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
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
    path: PathBuf,
    max: u16,
}

impl Device for Led {
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

    fn set(
        &self,
        value: Box<dyn AbsoluteBrightness<Number = Self::Number>>,
    ) -> Result<Self::Number, DeviceWriteError<Self::Number>> {
        let value = value
            .absolute_brightness(self)
            .ok_or(DeviceWriteError::NoValue)?;
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

        write!(file, "{value}")
            .map(|_| value)
            .map_err(DeviceWriteError::from)
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.path.clone())
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
            let max = fs::read_to_string(path.join("max_brightness"))
                .map_err(DeviceReadError::from)
                .and_then(|max| max.trim_end().parse().map_err(DeviceReadError::from))?;
            Ok(Self { path, max })
        } else {
            Err(NewLedError::NotEnoughFiles {
                relevant_files_count,
            })
        }
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
