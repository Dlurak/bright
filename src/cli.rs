use crate::brightness::Value;
use clap::{Parser, Subcommand, value_parser};
use std::{num::NonZero, time::Duration};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    #[command(alias = "ls", alias = "l")]
    List,
    #[command(aliases = ["metadata", "info"])]
    Meta {
        #[arg(long)]
        device: Option<String>,
    },
    Set(SetArgs),
}

#[derive(Parser, Clone)]
pub struct SetArgs {
    #[arg()]
    pub brightness: crate::restoration::SetValue,
    #[arg(long)]
    pub device: Option<String>,
    #[arg(short, long, group = "time")]
    pub duration: Option<humantime::Duration>,
    #[arg(short, long, alias = "change", group = "time")]
    pub change_per_second: Option<NonZero<u16>>,
    #[arg(
        long,
        requires = "time",
        default_value_t = 30,
        value_parser = value_parser!(u16).range(1..=1000)
    )]
    pub fps: u16,
    #[arg(long, default_value_t = Value::Absolute(0))]
    pub min: Value,
    #[arg(long, default_value_t = Value::Percentage(100))]
    pub max: Value,
    #[arg(long, default_value_t = false)]
    pub save: bool,
}

impl SetArgs {
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / u64::from(self.fps))
    }

    pub fn change_per_frame(&self, diff: NonZero<i32>) -> Result<NonZero<i32>, String> {
        if self.change_per_second.is_some_and(|ch| ch.get() < self.fps) {
            return Err(String::from(
                "Change per second must be at least equal to fps, try changing these values",
            ));
        }
        let change_from_change = self
            .change_per_second
            .map(|change_per_sec| change_per_sec.get() / self.fps);
        let change_from_duration = self.duration.map(|dur| {
            let frame_count = (dur.as_millis() / self.frame_duration().as_millis()).max(1) as u16;
            diff.unsigned_abs().get() as u16 / frame_count
        });

        let change = change_from_change
            .or(change_from_duration)
            .unwrap_or_else(|| {
                diff.unsigned_abs()
                    .get()
                    .try_into()
                    .expect("i32 -> u16 is save when it is unsigned")
            });

        let sign = if diff.is_positive() { 1 } else { -1 };
        // Safety: a != 0 <=> a/b != 0 and diff != 0
        let signed = unsafe { NonZero::new_unchecked(i32::from(change) * sign) };
        Ok(signed)
    }
}
