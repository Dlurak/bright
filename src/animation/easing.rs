use std::{
    fmt::{Display, Write},
    num::ParseFloatError,
    str::FromStr,
};
use delegate::delegate;
use thiserror::Error;

pub trait Easing {
    fn to_actual(&self, user_facing: f64) -> f64;
    #[allow(clippy::wrong_self_convention)]
    fn from_actual(&self, actual: f64) -> f64;
}

#[derive(Debug, Error)]
pub enum EasingParseError {
    #[error("invalid string pattern")]
    InvalidPattern,
    #[error("invalid number")]
    InvalidNum,
    #[error("can't parse float")]
    ParseFloat(
        #[source]
        #[from]
        ParseFloatError,
    ),
}

#[derive(Clone)]
pub struct Exponential {
    base: f64,
}

impl Exponential {
    fn try_new(base: f64) -> Option<Self> {
        (base != 1.0 && base > 0.0).then_some(Self { base })
    }
}

impl Easing for Exponential {
    fn to_actual(&self, user_facing: f64) -> f64 {
        self.base.powf(user_facing) - 1.0
    }

    fn from_actual(&self, actual: f64) -> f64 {
        (actual + 1.0).log(self.base)
    }
}

impl FromStr for Exponential {
    type Err = EasingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_string();
        match s.pop().zip(s.pop()) {
            Some(('x', '^')) => {
                let base = s.parse()?;
                Self::try_new(base).ok_or(EasingParseError::InvalidNum)
            }
            _ => Err(EasingParseError::InvalidPattern),
        }
    }
}

impl Display for Exponential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}^x", self.base)
    }
}

#[derive(Clone)]
pub struct Polynomial {
    exponent: f64,
}

impl Polynomial {
    fn try_new(exponent: f64) -> Option<Self> {
        (exponent > 0.0).then_some(Self { exponent })
    }
}

impl Easing for Polynomial {
    fn to_actual(&self, user_facing: f64) -> f64 {
        user_facing.powf(self.exponent)
    }

    fn from_actual(&self, actual: f64) -> f64 {
        actual.powf(1.0 / self.exponent)
    }
}

impl FromStr for Polynomial {
    type Err = EasingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next().zip(chars.next()) {
            Some(('x', '^')) => {
                let remainder: String = chars.collect();
                let exponent = remainder.parse()?;
                Self::try_new(exponent).ok_or(EasingParseError::InvalidNum)
            }
            _ => Err(EasingParseError::InvalidPattern),
        }
    }
}

impl Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x^{}", self.exponent)
    }
}

struct Linear;

impl Easing for Linear {
    fn to_actual(&self, user_facing: f64) -> f64 {
        user_facing
    }
    fn from_actual(&self, actual: f64) -> f64 {
        actual
    }
}

impl Display for Linear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('x')
    }
}

impl FromStr for Linear {
    type Err = EasingParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "x" {
            Ok(Self)
        } else {
            Err(EasingParseError::InvalidPattern)
        }
    }
}

#[derive(Default, Clone)]
pub enum EasingKind {
    #[default]
    Linear,
    Exponential(Exponential),
    Polynomial(Polynomial),
}

impl EasingKind {
    pub fn new_exponential(base: f64) -> Option<Self> {
        Exponential::try_new(base).map(Self::Exponential)
    }

    pub fn new_polynomial(exponent: f64) -> Option<Self> {
        Polynomial::try_new(exponent).map(Self::Polynomial)
    }
}

impl Display for EasingKind {
    delegate! {
        to match self {
            Self::Linear => Linear,
            Self::Exponential(exp) => exp,
            Self::Polynomial(pol) => pol,
        } {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
        }
    }
}

impl Easing for EasingKind {
    delegate! {
        to match self {
            Self::Linear => Linear,
            Self::Exponential(exp) => exp,
            Self::Polynomial(pol) => pol,
        } {
            fn to_actual(&self, user_facing: f64) -> f64;
            fn from_actual(&self, actual: f64) -> f64;
        }
    }
}

impl FromStr for EasingKind {
    type Err = EasingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse().map(Self::Exponential) {
            Err(EasingParseError::InvalidPattern) => {}
            result => return result,
        }

        match s.parse().map(Self::Polynomial) {
            Err(EasingParseError::InvalidPattern) => {}
            result => return result,
        }

        s.parse::<Linear>().map(|_| Self::Linear)
    }
}
