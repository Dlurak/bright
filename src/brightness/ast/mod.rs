pub mod functions;

use super::lexer::Token;
use super::lexer::{TokenCategory, UnsupportedCharError, lexer};
use crate::{
    animation::easing::Easing,
    device::{Device, errors::DeviceReadError},
};
use std::{iter::Peekable, path::PathBuf, str::FromStr};
use thiserror::Error;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChangeDirection {
    Inc,
    #[default]
    Abs,
    Dec,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ast {
    Literal {
        direction: ChangeDirection,
        value: u16,
        percent: bool,
    },
    Function {
        name: String,
        arguments: Vec<Ast>,
    },
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BrightnessEvaluationError {
    #[error("can't read the current brightness")]
    DeviceReadError(
        #[from]
        #[source]
        DeviceReadError,
    ),
    #[error("`{_0}` isn't a available function")]
    UnsupportedFunction(String),
    #[error(
        "`{function}` expects {} arguments but {provided} were provided",
        max.map_or_else(|| format!("at least {min}"), |n| format!("{min}-{n}"))
    )]
    WrongArgumentCount {
        function: String,
        provided: usize,
        min: usize, // Maybe replace these with `ArgumentCount` not totally sure
        max: Option<usize>,
    },
    #[error("file {} doesn't exist", _0.display())]
    MissingFile(PathBuf),
    #[error("a general error occured")]
    Other(
        #[source]
        #[from]
        Box<dyn std::error::Error>,
    ),
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseTokensError {
    #[error("no tokens given")]
    NoTokens,
    #[error(
        "{} expected `{}` but encountered `{}`",
        reason.as_ref().map_or(String::new(), |reason| format!("{reason}\n")),
        expected
            .as_ref()
            .map_or(
                "Nothing".to_string(),
                |(category, token)| {
                    token.as_ref().map_or(
                        format!("{category}"),
                        |token| format!("{token}")
                    )
                }
            ),
        encountered.name()
    )]
    IllegalToken {
        expected: Option<(TokenCategory, Option<Token>)>,
        encountered: Token,
        reason: Option<String>,
    },
    #[error("Unclosed delimiter")]
    UnclosedDelimiter,
}

impl Ast {
    pub fn evaluate(
        &self,
        device: &dyn Device,
        easing: &dyn Easing,
    ) -> Result<u16, BrightnessEvaluationError> {
        let current = device.current()?;

        match self {
            Self::Literal {
                direction,
                value,
                percent: true,
            } => {
                let max = f64::from(device.max());

                let current = easing.from_actual(f64::from(current) / max);
                let value = f64::from(*value) / 100.0;

                let new_perceived = match direction {
                    ChangeDirection::Inc => (current + value).clamp(0.0, 1.0),
                    ChangeDirection::Dec => (current - value).clamp(0.0, 1.0),
                    ChangeDirection::Abs => value,
                };
                let new_actual = easing.to_actual(new_perceived);
                Ok((new_actual * max) as u16)
            }
            Self::Literal {
                direction,
                value,
                percent: false,
            } => {
                let max = device.max();
                let value = *value;

                Ok(match direction {
                    ChangeDirection::Inc => current.saturating_add(value).min(max),
                    ChangeDirection::Dec => current.saturating_sub(value),
                    ChangeDirection::Abs => value,
                })
            }
            Self::Function { name, arguments } => {
                let Some(f) = functions::get_function(name.as_str()) else {
                    return Err(BrightnessEvaluationError::UnsupportedFunction(
                        name.to_string(),
                    ));
                };

                let expected_count = f.argument_count();
                if !expected_count.valid(arguments.len()) {
                    return Err(BrightnessEvaluationError::WrongArgumentCount {
                        function: f.name().to_string(),
                        provided: arguments.len(),
                        min: expected_count.min,
                        max: expected_count.max,
                    });
                }

                f.call(arguments, device, easing)
            }
        }
    }

    pub fn parse_tokens<I>(tokens: &mut Peekable<I>) -> Result<Self, ParseTokensError>
    where
        I: Iterator<Item = Token>,
    {
        match tokens.next().ok_or(ParseTokensError::NoTokens)? {
            Token::Number(value) => {
                let percent = matches!(tokens.peek(), Some(Token::Percent));
                if percent {
                    tokens.next();
                }

                let direction = match tokens.peek() {
                    Some(Token::Plus) => ChangeDirection::Inc,
                    Some(Token::Minus) => ChangeDirection::Dec,
                    _ => ChangeDirection::default(),
                };

                Ok(Self::Literal {
                    direction,
                    value,
                    percent,
                })
            }
            Token::Identifier(name) => {
                match tokens.peek() {
                    None => {
                        // identifier without () → treat as zero-arg function
                        return Ok(Self::Function {
                            name,
                            arguments: vec![],
                        });
                    }
                    Some(Token::LeftParentheses) => {
                        tokens.next(); // consume '(' and continue on
                    }
                    Some(encountered) => {
                        return Err(ParseTokensError::IllegalToken {
                            expected: Some((
                                Token::LeftParentheses.into(),
                                Some(Token::LeftParentheses),
                            )),
                            encountered: encountered.clone(),
                            reason: Some("Functions must be called".to_string()),
                        });
                    }
                }

                let mut arguments = Vec::new();

                let mut indent_level = 1;
                let mut arg_tokens = Vec::new();

                while indent_level >= 1 {
                    let tok = tokens.next().ok_or(ParseTokensError::UnclosedDelimiter)?; // `?` for missing ')'

                    match tok {
                        Token::LeftParentheses => indent_level += 1,
                        Token::RightParentheses => indent_level -= 1,
                        _ => {}
                    }

                    if indent_level == 0 {
                        break;
                    }

                    if tok == Token::Comma {
                        arguments.push(Self::parse_tokens(&mut arg_tokens.into_iter().peekable())?);
                        arg_tokens = Vec::new();
                    } else {
                        arg_tokens.push(tok);
                    }
                }

                if !(arguments.is_empty() && arg_tokens.is_empty()) {
                    // for zero argument functions
                    arguments.push(Self::parse_tokens(&mut arg_tokens.into_iter().peekable())?);
                }

                Ok(Self::Function { name, arguments })
            }
            tok => Err(ParseTokensError::IllegalToken {
                expected: Some((TokenCategory::Standalone, None)),
                encountered: tok,
                reason: Some("SERAFIN".to_string()),
            }),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AstFromStrError {
    #[error("{_0}")]
    LexerError(
        #[from]
        #[source]
        UnsupportedCharError,
    ),
    #[error("{_0}")]
    TokenParseError(
        #[from]
        #[source]
        ParseTokensError,
    ),
}

impl FromStr for Ast {
    type Err = AstFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = lexer(s)?;
        let ast = Ast::parse_tokens(&mut tokens.into_iter().peekable())?;
        Ok(ast)
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::Function {
            name: "current".to_string(),
            arguments: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{animation::easing::EasingKind, meta::Meta};

    struct TestDevice {
        max: u16,
        current: u16,
    }

    impl Meta for TestDevice {
        fn meta(&self, _: &crate::config::Easings) -> Vec<crate::meta::Information> {
            vec![]
        }
    }

    impl Device for TestDevice {
        fn name(&self) -> Option<&str> {
            Some("TestDevice")
        }

        fn max(&self) -> u16 {
            self.max
        }

        fn current(&self) -> Result<u16, crate::device::errors::DeviceReadError> {
            Ok(self.current)
        }

        fn set(&self, value: u16) -> Result<u16, crate::device::errors::DeviceWriteError<u16>> {
            Ok(value.min(self.max))
        }
    }

    #[test]
    fn test_ast_evaluation() {
        assert_eq!(
            "current"
                .parse::<Ast>()
                .unwrap()
                .evaluate(
                    &TestDevice {
                        max: 1_000,
                        current: 500,
                    },
                    &EasingKind::Linear
                )
                .unwrap(),
            500
        );

        assert_eq!(
            "100+"
                .parse::<Ast>()
                .unwrap()
                .evaluate(
                    &TestDevice {
                        max: 1_000,
                        current: 500,
                    },
                    &EasingKind::Linear
                )
                .unwrap(),
            600
        );

        assert_eq!(
            "100+"
                .parse::<Ast>()
                .unwrap()
                .evaluate(
                    &TestDevice {
                        max: 1_000,
                        current: 500,
                    },
                    &EasingKind::Linear
                )
                .unwrap(),
            600
        );

        assert_eq!(
            "clamp(20, 200+, 90%)"
                .parse::<Ast>()
                .unwrap()
                .evaluate(
                    &TestDevice {
                        max: 1_000,
                        current: 500,
                    },
                    &EasingKind::Linear
                )
                .unwrap(),
            700
        );

        assert_eq!(
            "clamp(20, 200+, 90%)"
                .parse::<Ast>()
                .unwrap()
                .evaluate(
                    &TestDevice {
                        max: 1_000,
                        current: 800,
                    },
                    &EasingKind::Linear
                )
                .unwrap(),
            900
        );
    }

    #[test]
    fn test_ast_fail() {
        let dev = TestDevice {
            max: 1_000,
            current: 800,
        };
        assert!(
            "clamp"
                .parse::<Ast>()
                .unwrap()
                .evaluate(&dev, &EasingKind::Linear)
                .is_err()
        );

        assert!(
            "never_existing"
                .parse::<Ast>()
                .unwrap()
                .evaluate(&dev, &EasingKind::Linear)
                .is_err()
        );

        assert!(
            "current(20)"
                .parse::<Ast>()
                .unwrap()
                .evaluate(&dev, &EasingKind::Linear)
                .is_err()
        );
    }

    #[test]
    fn test_ast_generator() {
        assert_eq!(
            "current".parse::<Ast>().unwrap(),
            Ast::Function {
                name: "current".to_string(),
                arguments: vec![]
            }
        );

        assert_eq!(
            "clamp(50%,    current(), 20-)".parse::<Ast>().unwrap(),
            Ast::Function {
                name: "clamp".to_string(),
                arguments: vec![
                    Ast::Literal {
                        direction: ChangeDirection::Abs,
                        value: 50,
                        percent: true
                    },
                    Ast::Function {
                        name: "current".to_string(),
                        arguments: vec![]
                    },
                    Ast::Literal {
                        direction: ChangeDirection::Dec,
                        value: 20,
                        percent: false
                    },
                ]
            }
        );

        assert_eq!(
            "42+".parse::<Ast>().unwrap(),
            Ast::Literal {
                direction: ChangeDirection::Inc,
                value: 42,
                percent: false
            }
        );

        assert_eq!(
            "100%".parse::<Ast>().unwrap(),
            Ast::Literal {
                direction: ChangeDirection::Abs,
                value: 100,
                percent: true
            }
        );

        assert_eq!(
            "4%-".parse::<Ast>().unwrap(),
            Ast::Literal {
                direction: ChangeDirection::Dec,
                value: 4,
                percent: true
            }
        );

        assert_eq!(
            "".parse::<Ast>().unwrap_err(),
            AstFromStrError::TokenParseError(ParseTokensError::NoTokens)
        );

        assert_eq!(
            "cl@mp()".parse::<Ast>().unwrap_err(),
            AstFromStrError::LexerError(UnsupportedCharError {
                char: '@',
                index: 2
            })
        );

        assert_eq!(
            "clamp(((((())".parse::<Ast>().unwrap_err(),
            AstFromStrError::TokenParseError(ParseTokensError::UnclosedDelimiter)
        );

        assert!(matches!(
            "max 2".parse::<Ast>().unwrap_err(),
            AstFromStrError::TokenParseError(ParseTokensError::IllegalToken {
                expected: Some(_),
                encountered: Token::Number(2),
                reason: _,
            })
        ));

        assert!(matches!(
            "clamp(20, 20,)".parse::<Ast>().unwrap_err(),
            AstFromStrError::TokenParseError(ParseTokensError::NoTokens)
        ));
    }
}
