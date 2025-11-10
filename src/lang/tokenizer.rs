//! ## Tokenizer
//!
//! Split a source text into a stream of Tokens (or errors, if text contains invalid tokens).
//!
//! Test implementation for now; will modify to tokenize full grammer later.
//!
//! TODO:
//! ---
//!
//! - [x] grammar spec
//! - [ ] as numbered productions
//! - [ ] lexer/tokenizer
//! - [ ] parser
//! - [ ] ast def
use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::LazyLock,
};

use regex::Regex;

/// Convert a given source text into an iterator of `Token`s, ready to be parsed.
pub fn tokenize(text: &str) -> TokenIter {
    TokenIter::from(text)
}

#[derive(Debug)]
pub enum Token {
    /// Integers
    Int(isize),
    // /// Binary operators
    // /// - EqOp
    // /// - LteOp
    // /// - GtOp
    // /// - LtOp
    // /// - PlusOp
    // /// - MinusOp
    // /// - MultOp
    // /// - DivOp
    // /// Parenthises
    // /// - OParen
    // /// - CParen
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(s), Self::Int(o)) => s == o,
            _ => false,
        }
    }
}

impl From<(usize, &str)> for Token {
    fn from((idx, val): (usize, &str)) -> Self {
        match idx {
            // Int matcher
            0 => Self::Int(val.parse().expect("{val} should match isize at this point")),
            _ => panic!("this should never happen"),
        }
    }
}

static MATCHERS: LazyLock<[Regex; 1]> = LazyLock::new(|| {
    [
        // Int matcher
        Regex::new(r"^-?[0-9]+").expect("hardcoded int regex should compile."),
    ]
});

fn take_from<'a>(
    text: &str,
    mut matchers: impl Iterator<Item = &'a Regex>,
) -> Option<(Token, usize)> {
    matchers
        .enumerate()
        .find_map(|(idx, re)| {
            #[cfg(test)]
            {
                println!("[tokenizer::take_from] searching '{text}' using matcher {idx}: {re:?}");
            }
            re.find(text).and_then(|m| Some((idx, m.as_str())))
        })
        .map(|found| (Token::from(found), found.1.chars().count()))
}

pub struct TokenIter<'a> {
    src: &'a str,
    end: usize,
    pos: usize,
    err: bool,
}

impl<'a> TokenIter<'a> {
    pub fn new(src: &'a str, end: usize) -> Self {
        Self {
            src,
            end,
            pos: 0,
            err: false,
        }
    }
}

impl<'a> From<&'a str> for TokenIter<'a> {
    fn from(value: &'a str) -> Self {
        let end = value.chars().count();

        Self::new(value, end)
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token, TokenErr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.err || self.pos >= self.end {
            // if error encountered
            // or if out of source text
            // not able to return more tokens
            None
        } else {
            // otherwise try to take token from the front
            match take_from(&self.src[self.pos..self.end], MATCHERS.iter()) {
                Some((tok, len)) => {
                    // if token successfully taken, update cursor position for next search
                    self.pos += len;
                    // then return token
                    Some(Ok(tok))
                }
                None => {
                    // update error status
                    self.err = true;
                    // then return error
                    Some(Err(TokenErr::Invalid(self.pos)))
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum TokenErr {
    Invalid(usize),
}

impl Display for TokenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::Invalid(pos) => format!("Invalid token at {pos}"),
        };

        write!(f, "TokenErr: {msg}")
    }
}
impl Error for TokenErr {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("1", vec![Token::Int(1)])]
    fn tokenize_inputs(#[case] text: &str, #[case] exp: Vec<Token>) {
        let tokens: Result<Vec<Token>, TokenErr> = tokenize(text).collect();
        let err_msg =
            format!("input '{text}' should tokenize successfully, instead got {tokens:?}");
        let act = tokens.expect(&err_msg);

        assert_eq!(act, exp)
    }
}
