use crate::{animation::easing::EasingKind, brightness::Value};
use clap::{Parser, Subcommand, value_parser};
use std::{num::NonZero, time::Duration};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    /// The easing to map perceived brightness → actual device brightness
    #[arg(
        long,
        long_help = "The easing to use\nIt maps perceived brightness to the actual brightness, both input and output should be in the interval 0.0..=1.0\nValid inputs look like: `x^2.5` (polynomial), `3.141^x` (exponential) or simply `x` (linear)"
    )]
    pub easing: Option<EasingKind>,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// List all found devices and show basic information about them
    #[command(alias = "ls", alias = "l")]
    List,
    /// Show detailed metadata about a selected device
    #[command(aliases = ["metadata", "info"])]
    Meta {
        /// Choose a device by name
        #[arg(
            long,
            long_help = "Choose a device by name\nThis takes presidence over the BRIGHT_DEVICE environment variable"
        )]
        device: Option<String>,
    },
    /// Change the brightness of a selected device
    Set(SetArgs),
}

#[derive(Parser, Clone)]
pub struct SetArgs {
    /// The new brightness to apply
    #[arg(
        long_help = "The new brightness to apply\nUsing the value `restore` you can restore the last saved brightness\nBoth absolute values and percentages are accepted, which both can be followed by an optional `+` or `-` to increase/decrease"
    )]
    pub brightness: crate::restoration::SetValue,
    /// Choose a device by name
    #[arg(
        long,
        long_help = "Choose a device by name\nThis takes presidence over the BRIGHT_DEVICE environment variable"
    )]
    pub device: Option<String>,
    /// The duration of the animation, if omitted the change isn't animated
    #[arg(short, long, group = "time")]
    pub duration: Option<humantime::Duration>,
    /// The fps (frames-per-second) the animation will have
    #[arg(
        long,
        requires = "time",
        default_value_t = 30,
        value_parser = value_parser!(u16).range(1..=1000)
    )]
    pub fps: u16,
    /// The minimum value which can be set, both absolute values and percentages are available
    #[arg(long, default_value_t = Value::Absolute(0))]
    pub min: Value,
    /// The maximum value which can be set, both absolute values and percentages are available
    #[arg(long, default_value_t = Value::Percentage(100.0))]
    pub max: Value,
    /// Whether to save the new value to a temporary file so it can be restored later on
    #[arg(long, default_value_t = false)]
    pub save: bool,
}

impl SetArgs {
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / u64::from(self.fps))
    }

    pub fn frame_count(&self) -> NonZero<usize> {
        self.duration
            .map(|dur| (dur.as_millis() / self.frame_duration().as_millis()).max(1) as usize)
            .and_then(NonZero::new)
            .unwrap_or(NonZero::new(1).unwrap())
    }
}
