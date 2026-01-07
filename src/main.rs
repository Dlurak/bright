mod animation;
mod brightness;
mod cli;
mod config;
mod device;
mod meta;

use crate::{
    animation::{AnimationIter, easing::Easing},
    brightness::ast::functions::restoration::write_brightness,
    cli::{Args, Command, SetArgs},
    config::{EasingFromFileError, Easings, MultilineEasingsParseError},
    device::{UNNAMED, all_devices, errors::DeviceWriteError, get_device},
};
use clap::Parser;
use std::{fmt::Write, process};

const UNDERLINE_FMT: &str = "\x1B[4m";
const DEFAULT_FMT: &str = "\x1B[0m";

fn main() {
    let Args { easing, command } = Args::parse();

    let easings = easing
        .map(Easings::from)
        .or_else(|| match Easings::from_config() {
            Ok(val) => val,
            Err(err) => {
                let msg = match err {
                    EasingFromFileError::NoPath => {
                        String::from("A path for the config file could not be determined")
                    }
                    EasingFromFileError::ParseError {
                        path,
                        error: MultilineEasingsParseError::ParseError { line_number, error },
                    } => format!(
                        "Can't parse easing in {}:{line_number}: {error}",
                        path.display()
                    ),
                    EasingFromFileError::ParseError {
                        path,
                        error:
                            MultilineEasingsParseError::DuplicateDevice {
                                line_number,
                                device,
                            },
                    } => format!(
                        "Config file {} has duplicated device {device} at line {line_number}",
                        path.display()
                    ),
                    EasingFromFileError::ReadFile(ref err) => {
                        format!("Can't read config file: {err}")
                    }
                };

                eprintln!("{msg}");
                process::exit(1);
            }
        })
        .unwrap_or_default();

    let result = match command {
        Command::List => {
            list_handler(easings);
            Ok(())
        }
        Command::Meta { device } => meta_handler(device, easings),
        Command::Set(args) => set_handler(args, easings),
    };
    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1)
    }
}

fn list_handler(easings: Easings) {
    for (class, devices) in all_devices() {
        println!("{UNDERLINE_FMT}{class}{DEFAULT_FMT}:");
        for device in devices {
            let cur = device.current().ok();
            let max = device.max();

            let easing = easings.get_or_default(device.name());

            let name = device.name().unwrap_or(UNNAMED);
            print!("\t{name}");

            if let Some(path) = device.path() {
                print!(" {}", path.display());
            }

            print!(
                " {}/{max}",
                cur.map_or_else(|| String::from('?'), |n| n.to_string())
            );

            if let Some(cur) = cur {
                let actual = f64::from(cur) / f64::from(max);
                let user_facing = easing.from_actual(actual);
                let perc = user_facing * 100.0;
                println!(" ({perc}%)");
            } else {
                println!();
            }
        }
    }
}

fn meta_handler(device_name: Option<String>, easings: Easings) -> Result<(), String> {
    let device = get_device(device_name).map_err(|err| err.to_string())?;

    for info in device.meta(&easings) {
        println!("{info}");
    }

    Ok(())
}

fn set_handler(args: SetArgs, easings: Easings) -> Result<(), String> {
    let device = get_device(args.device.as_deref()).map_err(|err| err.to_string())?;
    let name = device.name();
    let easing = easings.get_or_default(name);
    let name = name.unwrap_or(UNNAMED);

    println!("Updating device: '{name}'");

    let prev_brightness = device
        .current()
        .map_err(|err| format!("Reading current brightness: {err}"))?;

    let desired_brightness = args
        .brightness
        .evaluate(&*device, &easing)
        .map_err(|err| format!("While determening the brightness encountered an error: {err}"))?;

    if args.save {
        let path = write_brightness(name, prev_brightness).map_err(|err| err.to_string())?;
        println!(
            "Wrote previous brightness of {prev_brightness} to {}",
            path.display()
        );
    }

    if i32::from(prev_brightness) == i32::from(desired_brightness) {
        println!("Already at the desired brightness of {desired_brightness}");
        return Ok(());
    };

    println!("Previously: {prev_brightness}");

    let mut last_applied = None;
    let animation_values = AnimationIter::new(
        (prev_brightness, desired_brightness),
        device.max(),
        args.frame_count(),
        easing,
    );
    for (brightness, is_last) in animation_values {
        match device.set(brightness) {
            Ok(new) => {
                last_applied = Some(new);
                println!("Updated: {new}");
            }
            Err(DeviceWriteError::Write(err)) => {
                let kind = err.kind();

                let mut buffer = format!("Error: {kind}");
                if let Some(os_error) = err.raw_os_error() {
                    write!(buffer, "\nOS-Error: {os_error}")
                        .expect("Writing into String is infallible");
                }

                if kind == std::io::ErrorKind::PermissionDenied {
                    buffer.push_str("\nTipp: Set an udev rule or run with elevated priviliges");
                    return Err(buffer);
                }
                eprintln!("{buffer}");
            }
            Err(DeviceWriteError::Overflow { max, provided }) => {
                return Err(format!(
                    "Tried setting the brightness to {provided} eventhough only {max} is supported"
                ));
            }
        }

        if !is_last {
            std::thread::sleep(args.frame_duration());
        }
    }

    let actual_brightness = last_applied.unwrap_or(prev_brightness);
    println!("Finished: {actual_brightness}");
    Ok(())
}
