use std::fmt::Display;

pub mod animation;
pub mod brightness;
pub mod cli;
pub mod config;
pub mod device;
pub mod meta;

pub fn fmt_option<O, D>(opt: Option<O>, default: D) -> String
where
    O: Display,
    D: Display,
{
    opt.map_or_else(|| default.to_string(), |n| n.to_string())
}
