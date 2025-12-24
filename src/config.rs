use crate::animation::easing::{EasingKind, EasingParseError};
use derive_more::Display;
use std::{
    collections::{HashMap, hash_map::Entry},
    env, fs, io,
    path::PathBuf,
    str::FromStr,
};
use thiserror::Error;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Display)]
pub enum EasingDevice {
    #[display("default")]
    Default,
    #[display("{_0}")]
    Name(String),
}

impl From<&str> for EasingDevice {
    fn from(value: &str) -> Self {
        Self::Name(value.to_string())
    }
}

pub struct Easings(HashMap<EasingDevice, EasingKind>);

impl Easings {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_config() -> Result<Option<Self>, EasingFromFileError> {
        let Some(path) = path() else {
            return Err(EasingFromFileError::NoPath);
        };

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;

        match Self::from_str(&content) {
            Ok(easings) => Ok(Some(easings)),
            Err(error) => Err(EasingFromFileError::ParseError { path, error }),
        }
    }

    pub fn get_or_default<S: ToString>(&self, name: Option<S>) -> EasingKind {
        let easing_device = name.map_or(EasingDevice::Default, |name| {
            EasingDevice::Name(name.to_string())
        });

        self.0
            .get(&easing_device)
            .or_else(|| self.0.get(&EasingDevice::Default))
            .copied()
            .unwrap_or_default()
    }
}

impl Default for Easings {
    fn default() -> Self {
        let mut hm = HashMap::new();
        hm.insert(EasingDevice::Default, EasingKind::default());
        Self(hm)
    }
}

impl From<EasingKind> for Easings {
    fn from(value: EasingKind) -> Self {
        let mut hm = HashMap::new();
        hm.insert(EasingDevice::Default, value);
        Self(hm)
    }
}

#[derive(Debug, Error)]
pub enum EasingFromFileError {
    #[error("no config file path could be found")]
    NoPath,
    #[error("could not read file")]
    ReadFile(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("could not parse file")]
    ParseError {
        path: PathBuf,
        #[source]
        error: MultilineEasingsParseError,
    },
}

fn path() -> Option<PathBuf> {
    if let Ok(path) = env::var("BRIGHT_CONFIG") {
        let path = PathBuf::from(path);
        return if path.is_file() { Some(path) } else { None };
    }

    let config_dir = dirs::home_dir()
        .map(|home| home.join(".config"))
        .or_else(dirs::config_dir)?;
    let config_file = config_dir.join("bright").join("easings");

    if config_file.is_file() {
        Some(config_file)
    } else {
        None
    }
}

#[derive(Debug, Error)]
pub enum MultilineEasingsParseError {
    #[error("can't parse line {l}", l = line_number + 1)]
    ParseError {
        line_number: usize,
        #[source]
        error: EasingParseError,
    },
    #[error("device {device} is duplicated in line {l}", l = line_number + 1)]
    DuplicateDevice {
        line_number: usize,
        device: EasingDevice,
    },
}

impl FromStr for Easings {
    type Err = MultilineEasingsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut easings = Self::new();

        for (i, line) in s.lines().enumerate() {
            let line = line.trim();
            let (dev, easing) = match line.split_once('=') {
                // only trim the middle as the line itself is already trimmed
                Some((dev_name, easing)) => {
                    (EasingDevice::from(dev_name.trim_end()), easing.trim_start())
                }
                None => (EasingDevice::Default, line),
            };

            let easing = EasingKind::from_str(easing).map_err(|error| {
                MultilineEasingsParseError::ParseError {
                    line_number: i,
                    error,
                }
            })?;

            match easings.0.entry(dev) {
                Entry::Vacant(entry) => {
                    entry.insert(easing);
                }
                Entry::Occupied(entry) => {
                    return Err(MultilineEasingsParseError::DuplicateDevice {
                        line_number: i,
                        device: entry.key().clone(), // only clone in case of error
                    });
                }
            }
        }

        Ok(easings)
    }
}
