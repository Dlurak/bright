use derive_more::Display;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Debug, Display)]
pub enum Token {
    LeftParentheses,
    RightParentheses,
    Percent,
    Comma,
    Plus,
    Minus,

    Number(u16),

    Identifier(String),
}

impl Token {
    fn new_atomic(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::LeftParentheses),
            ')' => Some(Self::RightParentheses),
            '%' => Some(Self::Percent),
            ',' => Some(Self::Comma),
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::LeftParentheses => "`(`",
            Self::RightParentheses => "`)`",
            Self::Percent => "`%`",
            Self::Comma => "`,`",
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Number(_) => "number",
            Self::Identifier(_) => "identifier",
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum TokenCategory {
    /// Tokens that can work on their own, numbers and identifiers
    #[display("Standalone")]
    Standalone,
    /// Tokens that can modify the exact meaning of a Standalone Token, for example `%`
    #[display("Supportive")]
    Supportive,
    /// Tokens which only have a symbolical meaning in the grammar
    #[display("Grammar")]
    Grammar,
}

impl From<Token> for TokenCategory {
    fn from(value: Token) -> Self {
        match value {
            Token::Number(_) | Token::Identifier(_) => Self::Standalone,
            Token::Percent | Token::Plus | Token::Minus => Self::Supportive,
            Token::Comma | Token::LeftParentheses | Token::RightParentheses => Self::Grammar,
        }
    }
}

#[derive(Error, Debug, PartialEq)]
#[error("`{char}` at {index} isn't supported")]
pub struct UnsupportedCharError {
    pub char: char,
    pub index: usize,
}

pub fn lexer<S>(str: S) -> Result<Vec<Token>, UnsupportedCharError>
where
    S: AsRef<str>,
{
    let mut tokens = Vec::new();
    let mut new_token_starts = true;

    for (i, c) in str.as_ref().chars().enumerate() {
        if let Some(atomic) = Token::new_atomic(c) {
            tokens.push(atomic);
        } else if let Some(digit) = c.to_digit(10) {
            match tokens.last_mut() {
                Some(Token::Number(last)) if !new_token_starts => {
                    *last = *last * 10 + digit as u16;
                }
                _ => tokens.push(Token::Number(digit as u16)),
            }
        } else if (c.is_ascii() && c.is_alphabetic()) || c == '_' {
            match tokens.last_mut() {
                Some(Token::Identifier(str)) if !new_token_starts => {
                    str.push(c);
                }
                _ => tokens.push(Token::Identifier(c.to_string())),
            }
        } else if !c.is_whitespace() {
            return Err(UnsupportedCharError { char: c, index: i });
        }

        new_token_starts = c.is_whitespace();
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        use Token as To;

        assert_eq!(lexer("").unwrap(), vec![]);

        assert_eq!(
            lexer("12 42").unwrap(),
            vec![To::Number(12), To::Number(42)]
        );
        assert_eq!(
            lexer("clamp(12, 20%, restore(), current(), 5%-)").unwrap(),
            vec![
                To::Identifier(String::from("clamp")),
                To::LeftParentheses,
                To::Number(12),
                To::Comma,
                To::Number(20),
                To::Percent,
                To::Comma,
                To::Identifier(String::from("restore")),
                To::LeftParentheses,
                To::RightParentheses,
                To::Comma,
                To::Identifier(String::from("current")),
                To::LeftParentheses,
                To::RightParentheses,
                To::Comma,
                To::Number(5),
                To::Percent,
                To::Minus,
                To::RightParentheses,
            ]
        );
    }
}
