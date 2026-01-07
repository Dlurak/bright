use super::{Ast, BrightnessEvaluationError};
use crate::{animation::easing::Easing, device::Device};

pub struct ArgumentCount {
    pub min: usize,
    pub max: Option<usize>,
}

impl ArgumentCount {
    #[inline]
    pub fn valid(&self, count: usize) -> bool {
        count >= self.min && self.max.is_none_or(|m| count <= m)
    }

    const fn new(min: usize, max: Option<usize>) -> Self {
        Self { min, max }
    }

    const fn exactly(count: usize) -> Self {
        Self {
            min: count,
            max: Some(count),
        }
    }

    const fn empty() -> Self {
        Self {
            min: 0,
            max: Some(0),
        }
    }
}

pub trait Function {
    fn name(&self) -> &'static str;
    fn argument_count(&self) -> ArgumentCount;

    fn call(
        &self,
        arguments: &[Ast],
        device: &dyn Device,
        easing: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError>;
}

pub struct Clamp;

impl Function for Clamp {
    fn name(&self) -> &'static str {
        "clamp"
    }
    fn argument_count(&self) -> ArgumentCount {
        ArgumentCount::exactly(3)
    }

    fn call(
        &self,
        arguments: &[Ast],
        device: &dyn Device,
        easing: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError> {
        let min = arguments[0].evaluate(device, easing)?;
        let optimal = arguments[1].evaluate(device, easing)?;
        let max = arguments[2].evaluate(device, easing)?;
        Ok(optimal.clamp(min, max))
    }
}

pub struct Current;

impl Function for Current {
    fn name(&self) -> &'static str {
        "current"
    }
    fn argument_count(&self) -> ArgumentCount {
        ArgumentCount::empty()
    }

    fn call(
        &self,
        _: &[Ast],
        device: &dyn Device,
        _: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError> {
        Ok(device.current()?)
    }
}

pub struct Max;

impl Function for Max {
    fn name(&self) -> &'static str {
        "max"
    }
    fn argument_count(&self) -> ArgumentCount {
        ArgumentCount::new(1, None)
    }

    fn call(
        &self,
        arguments: &[Ast],
        device: &dyn Device,
        easing: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError> {
        let mut arguments = arguments.iter().map(|ast| ast.evaluate(device, easing));

        let mut max = arguments
            .next()
            .expect("Specified to have at least one argument")?;
        for val in arguments {
            let val = val?;
            if val > max {
                max = val;
            }
        }

        Ok(max)
    }
}

pub struct Min;

impl Function for Min {
    fn name(&self) -> &'static str {
        "min"
    }
    fn argument_count(&self) -> ArgumentCount {
        ArgumentCount::new(1, None)
    }

    fn call(
        &self,
        arguments: &[Ast],
        device: &dyn Device,
        easing: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError> {
        let mut arguments = arguments.iter().map(|ast| ast.evaluate(device, easing));

        let mut min = arguments
            .next()
            .expect("Specified to have at least one argument")?;
        for val in arguments {
            let val = val?;
            if val < min {
                min = val;
            }
        }

        Ok(min)
    }
}

pub fn get_function(name: &str) -> Option<Box<dyn Function>> {
    match name {
        "current" => Some(Box::new(Current)),
        "clamp" => Some(Box::new(Clamp)),
        "max" => Some(Box::new(Max)),
        "min" => Some(Box::new(Min)),
        "restore" => Some(Box::new(restoration::Restore)),
        _ => None,
    }
}

pub mod restoration {
    use thiserror::Error;

    use super::{super::BrightnessEvaluationError, ArgumentCount, Function};
    use crate::device::UNNAMED;
    use std::error::Error as StdError;
    use std::{
        fs::{self, File, read_to_string},
        io::{self, ErrorKind, Write},
        path::PathBuf,
    };

    pub struct Restore;

    impl Function for Restore {
        fn name(&self) -> &'static str {
            "restore"
        }

        fn argument_count(&self) -> ArgumentCount {
            ArgumentCount::empty()
        }

        fn call(
            &self,
            _: &[crate::brightness::ast::Ast],
            device: &dyn crate::device::Device,
            _: &dyn crate::animation::easing::Easing,
        ) -> Result<u16, BrightnessEvaluationError> {
            let path = device_restore_path(device.name().unwrap_or(UNNAMED));
            let value = read_to_string(&path).map_err(|err| {
                if err.kind() == ErrorKind::NotFound {
                    BrightnessEvaluationError::MissingFile(path)
                } else {
                    BrightnessEvaluationError::Other(Box::new(err) as Box<dyn StdError>)
                }
            })?;

            value
                .parse()
                .map_err(|err| BrightnessEvaluationError::Other(Box::new(err) as Box<dyn StdError>))
        }
    }

    fn device_restore_path(device_name: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{}/{device_name}", env!("CARGO_PKG_NAME")))
    }

    pub fn write_brightness(device_name: &str, brightness: u16) -> Result<PathBuf, WriteError> {
        let path = device_restore_path(device_name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(WriteError::DirCreate)?;
        }

        let mut file = File::create(&path).map_err(WriteError::FileCreate)?;
        let content = brightness.to_string();
        let content = content.as_bytes();
        match file.write_all(content) {
            Ok(()) => Ok(path),
            Err(err) => Err(WriteError::FileWrite(err)),
        }
    }

    #[derive(Debug, Error)]
    pub enum WriteError {
        #[error("error at directory creation: {_0}")]
        DirCreate(#[source] io::Error),
        #[error("error at file creation: {_0}")]
        FileCreate(#[source] io::Error),
        #[error("error when writing to file: {_0}")]
        FileWrite(#[source] io::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_count_validity() {
        assert!(ArgumentCount::new(0, Some(0)).valid(0));
        assert!(ArgumentCount::new(2, None).valid(4));
    }
}
