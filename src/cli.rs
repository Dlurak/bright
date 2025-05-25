use crate::{animation::easing::EasingKind, brightness::Value};
use clap::{Parser, Subcommand, value_parser};
use std::{num::NonZero, time::Duration};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    #[arg(long, default_value_t = EasingKind::default())]
    pub easing: EasingKind,
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
    #[arg(
        long,
        requires = "time",
        default_value_t = 30,
        value_parser = value_parser!(u16).range(1..=1000)
    )]
    pub fps: u16,
    #[arg(long, default_value_t = Value::Absolute(0))]
    pub min: Value,
    #[arg(long, default_value_t = Value::Percentage(100.0))]
    pub max: Value,
    #[arg(long, default_value_t = false)]
    pub save: bool,
}

impl SetArgs {
    pub fn frame_duration(&self) -> Duration {
        Duration::from_millis(1000 / u64::from(self.fps))
    }

    pub fn frames(&self) -> NonZero<usize> {
        self.duration
            .map(|dur| (dur.as_millis() / self.frame_duration().as_millis()).max(1) as usize)
            .and_then(NonZero::new)
            .unwrap_or(NonZero::new(1).unwrap())
    }
}
