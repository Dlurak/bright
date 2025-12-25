use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};
use thiserror::Error;

// struct Restore;
// impl AbsoluteBrightness for Restore {
//     fn absolute_brightness(
//         &self,
//         device: &dyn crate::device::Device,
//         _: &dyn Easing,
//     ) -> Result<u16, AbsoluteBrightnessError> {
//         let path = device_restore_path(device.name().unwrap_or(UNNAMED));
//         let value = read_to_string(&path).map_err(|err| {
//             if err.kind() == ErrorKind::NotFound {
//                 AbsoluteBrightnessError::MissingFile(path)
//             } else {
//                 AbsoluteBrightnessError::Other(Box::new(err) as Box<dyn StdError>)
//             }
//         })?;
//         value
//             .parse()
//             .map_err(|err| AbsoluteBrightnessError::Other(Box::new(err) as Box<dyn StdError>))
//     }
// }


