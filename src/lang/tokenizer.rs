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
    /// Binary operators
    Op(Op),
    /// Parenthises
    Paren(Paren),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(s), Self::Int(o)) => s == o,
            (Self::Op(s), Self::Op(o)) => s == o,
            (Self::Paren(s), Self::Paren(o)) => s == o,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Op {
    Eq,
    Gte,
    Lte,
    Gt,
    Lt,
    Plus,
    Minus,
    Mult,
    Div,
}

#[derive(Debug, PartialEq)]
enum Paren {
    Open,
    Closed,
}

impl From<(usize, &str)> for Token {
    fn from((idx, val): (usize, &str)) -> Self {
        match idx {
            // Int matcher
            0 => Self::Int(val.parse().expect("{val} should match isize at this point")),
            // Ops
            1 => Self::Op(Op::Eq),
            2 => Self::Op(Op::Gte),
            3 => Self::Op(Op::Lte),
            4 => Self::Op(Op::Gt),
            5 => Self::Op(Op::Lt),
            6 => Self::Op(Op::Plus),
            7 => Self::Op(Op::Minus),
            8 => Self::Op(Op::Mult),
            9 => Self::Op(Op::Div),
            10 => Self::Paren(Paren::Open),
            11 => Self::Paren(Paren::Closed),
            _ => panic!("this should never happen"),
        }
    }
}

static MATCHERS: LazyLock<[Regex; 12]> = LazyLock::new(|| {
    [
        // 0: Int matcher
        Regex::new(r"^\s*(-?[0-9]+)").expect("hardcoded int regex should compile."),
        // Op matchers
        // 1: eq
        Regex::new(r"^\s*(==)").expect("hardcoded int regex should compile."),
        // 2: gte
        Regex::new(r"^\s*(>=)").expect("hardcoded int regex should compile."),
        // 3: lte
        Regex::new(r"^\s*(<=)").expect("hardcoded int regex should compile."),
        // 4: gt
        Regex::new(r"^\s*(>)").expect("hardcoded int regex should compile."),
        // 5: lt
        Regex::new(r"^\s*(<)").expect("hardcoded int regex should compile."),
        // 6: plus
        Regex::new(r"^\s*(\+)").expect("hardcoded int regex should compile."),
        // 7: minus
        Regex::new(r"^\s*(-)").expect("hardcoded int regex should compile."),
        // 8: mult
        Regex::new(r"^\s*(\*)").expect("hardcoded int regex should compile."),
        // 9: div
        Regex::new(r"^\s*(/)").expect("hardcoded int regex should compile."),
        // Paren matchers
        // 10: open paren
        Regex::new(r"^\s*(\()").expect("hardcoded int regex should compile."),
        // 11: closed paren
        Regex::new(r"^\s*(\))").expect("hardcoded int regex should compile."),
    ]
});

fn take_from<'a>(text: &str, matchers: impl Iterator<Item = &'a Regex>) -> Option<(Token, usize)> {
    matchers.enumerate().find_map(|(idx, re)| {
        #[cfg(test)]
        {
            println!("[tokenizer::take_from] searching '{text}' using matcher {idx}: {re:?}");
        }
        re.captures(text).and_then(|caps| {
            #[cfg(test)]
            {
                println!("[tokenizer::take_from] found capture groups {caps:?}");
            }
            let num_chars_used = caps
                .get(0)
                .expect("0th capture group always exists")
                .as_str()
                .chars()
                .count();
            let first = caps
                .get(1)
                .expect("1st capture group always exists")
                .as_str();

            Some((Token::from((idx, first)), num_chars_used))
        })
    })
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
            match take_from(&self.src[self.pos..self.end], (&MATCHERS).iter()) {
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
    #[case::simple_pos("1", vec![Token::Int(1)])]
    #[case::mult_pos("123456879", vec![Token::Int(123456879)])]
    #[case::simple_neg("-1", vec![Token::Int(-1)])]
    #[case::simple_eq("==", vec![Token::Op(Op::Eq)])]
    #[case::simple_gte(">=", vec![Token::Op(Op::Gte)])]
    #[case::simple_lte("<=", vec![Token::Op(Op::Lte)])]
    #[case::simple_gte(">", vec![Token::Op(Op::Gt)])]
    #[case::simple_lte("<", vec![Token::Op(Op::Lt)])]
    #[case::simple_lte("+", vec![Token::Op(Op::Plus)])]
    #[case::simple_lte("-", vec![Token::Op(Op::Minus)])]
    #[case::simple_lte("*", vec![Token::Op(Op::Mult)])]
    #[case::simple_lte("/", vec![Token::Op(Op::Div)])]
    #[case::simple_lte("(", vec![Token::Paren(Paren::Open)])]
    #[case::simple_lte(")", vec![Token::Paren(Paren::Closed)])]
    #[case::one_lead_space(" 1", vec![Token::Int(1)])]
    #[case::mult_lead_space("    1", vec![Token::Int(1)])]
    fn tokenize_one(#[case] text: &str, #[case] exp: Vec<Token>) {
        let tokens: Result<Vec<Token>, TokenErr> = tokenize(text).collect();
        let err_msg =
            format!("input '{text}' should tokenize successfully, instead got {tokens:?}");
        let act = tokens.expect(&err_msg);

        assert_eq!(act, exp)
    }

    #[test]
    fn tokenize_many() {
        let input = "25 == ( 10 * 20 - 100) / 4 > 20";
        let exp = vec![
            Token::Int(25),
            Token::Op(Op::Eq),
            Token::Paren(Paren::Open),
            Token::Int(10),
            Token::Op(Op::Mult),
            Token::Int(20),
            Token::Op(Op::Minus),
            Token::Int(100),
            Token::Paren(Paren::Closed),
            Token::Op(Op::Div),
            Token::Int(4),
            Token::Op(Op::Gt),
            Token::Int(20),
        ];

        let tokens: Result<Vec<Token>, TokenErr> = tokenize(input).collect();
        let err_msg =
            format!("input '{input}' should tokenize successfully, instead got {tokens:?}");
        let act = tokens.expect(&err_msg);

        assert_eq!(act, exp)
    }
}
