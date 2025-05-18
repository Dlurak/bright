use bright::{
    cli::{Args, Command},
    device::{Device, errors::DeviceWriteError},
};
use clap::Parser;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
enum DeviceNotFound {
    #[error("no device named '{name}' available")]
    NoNamed { name: String },
    #[error("no device available")]
    NoFound,
}

fn get_device(dev: Option<String>) -> Result<Box<dyn Device<Number = u16>>, DeviceNotFound> {
    let devices = bright::device::all_lights();
    #[allow(clippy::single_match_else)] // this is easier to read as a match
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

fn main() {
    let args = Args::parse();

    match args.command {
        Command::List => {
            for (class, devices) in bright::device::all_lights() {
                println!("\x1B[4m{class}\x1B[0m:"); // Underlined
                for device in devices {
                    let cur = device.current().ok();
                    let max = device.max();

                    let name = device.name().unwrap_or("Unnamed");
                    print!("\t{name}");

                    if let Some(path) = device.path() {
                        print!(" {}", path.display());
                    }
                    if cur.is_some() || max.is_some() {
                        print!(" {}/{}", fmt_option(cur, '?'), fmt_option(max, '?'));
                    }

                    if let Some((cur, max)) = cur.zip(max) {
                        let percentage = (f64::from(cur) / f64::from(max)) * 100.0;
                        println!(" ({percentage}%)");
                    } else {
                        println!();
                    }
                }
            }
        }
        Command::Meta { device } => {
            let device = match get_device(device) {
                Ok(d) => d,
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(1);
                }
            };
            match (device.name(), device.path()) {
                (Some(name), Some(path)) => println!("Device: '{name}': {}", path.display()),
                (Some(name), None) => println!("Device: '{name}'"),
                (None, Some(path)) => println!("Device: {}", path.display()),
                (None, None) => println!("Device: unknown"),
            }
            let cur = device.current().ok();
            let max = device.max();
            let perc = cur
                .zip(max)
                .map(|(cur, max)| (f64::from(cur) / f64::from(max)) * 100.0);

            println!(
                "Current Brightness: {} ({}%)",
                fmt_option(cur, '?'),
                fmt_option(perc, '?')
            );
            println!("Max Brightness: {}", fmt_option(max, '?'));
        }
        Command::Set { device, brightness } => {
            let device = match get_device(device) {
                Ok(d) => d,
                Err(DeviceNotFound::NoFound) => {
                    eprintln!("No device available");
                    std::process::exit(1);
                }
                Err(DeviceNotFound::NoNamed { name }) => {
                    eprintln!("No device named '{name}' available");
                    std::process::exit(1);
                }
            };

            let prev = device.current().unwrap_or_else(|err| {
                eprintln!("Reading current brightness: {err}");
                std::process::exit(1)
            });

            let exit_code = match device.set(Box::new(brightness)) {
                Ok(new) => {
                    let max = device.max().map(f64::from);

                    println!("Updated device '{}':", device.name().unwrap_or("Unnamed"));
                    if let Some(max) = max {
                        println!(
                            "Previously: {prev} ({}%)",
                            (f64::from(prev) / max * 100.0).round()
                        );
                        println!(
                            "Updated: {new} ({}%)",
                            (f64::from(new) / max * 100.0).round()
                        );
                    } else {
                        println!("Previously: {prev}",);
                        println!("Updated: {new}",);
                    }

                    0
                }
                Err(DeviceWriteError::Write(err)) => {
                    let kind = err.kind();
                    if kind == std::io::ErrorKind::PermissionDenied {
                        eprintln!("Permission denied to write to file");
                        eprintln!("Tipp: Set an udev rule or run with elevated priviliges");
                    } else {
                        eprintln!("Error: {kind}");
                    }

                    if let Some(os_error) = err.raw_os_error() {
                        eprintln!("OS-Error: {os_error}");
                    }

                    1
                }
                Err(DeviceWriteError::Overflow { max, provided }) => {
                    let name = device
                        .name()
                        .map_or_else(|| String::from("the selected device"), |n| format!("'{n}'"));
                    eprintln!(
                        "Tried setting the brightness to {provided} eventhough {max} is the maximum that {name} is capable of"
                    );
                    1
                }
                Err(DeviceWriteError::NoValue) => {
                    eprintln!("Can't get absolute value for the new brightness");
                    1
                }
            };
            std::process::exit(exit_code)
        }
    }
}

fn fmt_option<O, D>(opt: Option<O>, default: D) -> String
where
    O: Display,
    D: Display,
{
    opt.map_or_else(|| default.to_string(), |n| n.to_string())
}
