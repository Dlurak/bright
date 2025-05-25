use bright::{
    animation::{AnimationIter, easing::Easing},
    brightness::AbsoluteBrightness,
    cli::{Args, Command, SetArgs},
    device::{UNNAMED, all_devices, errors::DeviceWriteError, get_device},
    fmt_option,
    restoration::write_brightness,
};
use clap::Parser;
use std::fmt::Write;

fn main() {
    let Args { easing, command } = Args::parse();

    let result = match command {
        Command::List => {
            list_handler(easing);
            Ok(())
        }
        Command::Meta { device } => meta_handler(device, easing),
        Command::Set(args) => set_handler(args, easing),
    };
    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1)
    }
}

fn list_handler(easing: impl Easing) {
    for (class, devices) in all_devices() {
        println!("\x1B[4m{class}\x1B[0m:"); // Underlined
        for device in devices {
            let cur = device.current().ok();
            let max = device.max();

            let name = device.name().unwrap_or(UNNAMED);
            print!("\t{name}");

            if let Some(path) = device.path() {
                print!(" {}", path.display());
            }
            if cur.is_some() || max.is_some() {
                print!(" {}/{}", fmt_option(cur, '?'), fmt_option(max, '?'));
            }

            if let Some((cur, max)) = cur.zip(max) {
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

fn meta_handler(device_name: Option<String>, easing: impl Easing) -> Result<(), String> {
    let device = get_device(device_name).map_err(|err| err.to_string())?;

    for info in device.meta(&easing) {
        println!("{info}");
    }

    Ok(())
}

fn set_handler(args: SetArgs, easing: impl Easing) -> Result<(), String> {
    let device = get_device(args.device.as_deref()).map_err(|err| err.to_string())?;
    let name = device.name().unwrap_or(UNNAMED);

    println!("Updating device: '{name}'");

    let prev_brightness = device
        .current()
        .map_err(|err| format!("Reading current brightness: {err}"))?;

    if args.save {
        let path = write_brightness(name, prev_brightness).map_err(|err| err.to_string())?;
        println!(
            "Wrote previous brightness of {prev_brightness} to {}",
            path.display()
        )
    }

    let min = args
        .min
        .absolute_brightness(&*device, &easing)
        .map_err(|err| {
            format!("While tetermening the minimum brightness encountered an error: {err}")
        })?;
    let max = args
        .max
        .absolute_brightness(&*device, &easing)
        .map_err(|err| {
            format!("While determening the maximum brightness encountered an error: {err}")
        })?;

    let original_brightness = args
        .brightness
        .absolute_brightness(&*device, &easing)
        .map_err(|err| format!("While determening the brightness encountered an error: {err}"))?;
    let desired_brightness = original_brightness.clamp(min, max);

    if original_brightness < min {
        println!("Desired brightness too low, applying minimum: {min}");
    } else if original_brightness > max {
        println!("Desired brightness too high, applying maximum: {max}");
    }

    if i32::from(prev_brightness) == i32::from(desired_brightness) {
        println!("Already at the desired brightness of {desired_brightness}");
        return Ok(());
    };

    println!("Previously: {prev_brightness}");

    let mut last_applied = None;
    let animation_values = AnimationIter::new(
        (prev_brightness, desired_brightness),
        max,
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
