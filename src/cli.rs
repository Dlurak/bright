use crate::brightness::{BrightnessValue, brightness_parser};
use clap::{Parser, Subcommand};

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
        #[arg(short, long)]
        device: Option<String>,
    },
    Set {
        #[arg(short, long)]
        device: Option<String>,
        #[arg(value_parser = brightness_parser)]
        brightness: BrightnessValue,
    },
}
