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
};

use crate::fun_tools::{Applicative, Functor, Monad};

type TokenResult = Result<Token, TokenErr>;

/// Convert a given source text into an iterator of `Token`s, ready to be parsed.
pub fn tokenize(text: &str) -> impl Iterator<Item = TokenResult> {
    vec![].into_iter()
}

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Integers
    Int(isize),
    /// Binary operators
    EqOp,
    GteOp,
    LteOp,
    GtOp,
    LtOp,
    PlusOp,
    MinusOp,
    MultOp,
    DivOp,
    /// Parenthises
    OParen,
    CParen,
}

struct Parser<T>(String);

trait State<'a, S, A> {
    type Output: Monad<'a, (S, A)>
    where
        S: 'a,
        A: 'a;

    fn run_state(state: S) -> Self::Output;
}

impl<'a, T> State<'a, String, T> for Parser<T> {
    type Output
        = Result<Option<(String, T)>, TokenErr>
    where
        String: 'a,
        T: 'a;
}

impl<'a, T: 'a, E: 'a> Applicative<'a, T> for Result<T, E> {
    type AHigherSelf<S: 'a> = Result<S, E>;

    fn pure(val: T) -> Self {
        Ok(val)
    }

    fn apply<B, F: Fn(&'a T) -> B>(&'a self, fs: Self::AHigherSelf<F>) -> Self::AHigherSelf<B> {
        match fs {
            Ok(f) => self.map(f),
            _ => self,
        }
    }
}

impl<'a, T: 'a, E: 'a> Monad<'a, T> for Result<T, E> {
    type MHigherType<S: 'a> = Result<S, E>;

    fn ret(val: T) -> Self
    where
        Self: Sized,
    {
        Ok(val)
    }

    fn bind<B: 'a, F: Fn(T) -> Self::MHigherType<B>>(self, f: F) -> Self::MHigherType<B> {
        self.and_then(f)
    }

    fn seq<B: 'a>(self, next: Self::MHigherType<B>) -> Self::MHigherType<B> {
        self.and(next)
    }
}

// pub struct TokenIter<'a> {
//     src: &'a str,
//     end: usize,
//     pos: usize,
//     err: bool,
// }
//
// impl<'a> TokenIter<'a> {
//     pub fn new(src: &'a str, end: usize) -> Self {
//         Self {
//             src,
//             end,
//             pos: 0,
//             err: false,
//         }
//     }
// }
//
// impl<'a> From<&'a str> for TokenIter<'a> {
//     fn from(value: &'a str) -> Self {
//         let end = value.chars().count();
//
//         Self::new(value, end)
//     }
// }
//
// impl<'a> Iterator for TokenIter<'a> {
//     type Item = Result<Token, TokenErr>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.err || self.pos >= self.end {
//             // if error encountered
//             // or if out of source text
//             // not able to return more tokens
//             None
//         } else {
//             // otherwise try to take token from the front
//             match take_from(&self.src[self.pos..self.end], (&MATCHERS).iter()) {
//                 Some((tok, len)) => {
//                     // if token successfully taken, update cursor position for next search
//                     self.pos += len;
//                     // then return token
//                     Some(Ok(tok))
//                 }
//                 None => {
//                     // update error status
//                     self.err = true;
//                     // then return error
//                     Some(Err(TokenErr::Invalid(self.pos)))
//                 }
//             }
//         }
//     }
// }

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
    // #[case::mult_pos("123456879", vec![Token::Int(123456879)])]
    // #[case::simple_neg("-1", vec![Token::Int(-1)])]
    // #[case::simple_eq("==", vec![Token::EqOp])]
    // #[case::simple_gte(">=", vec![Token::GteOp])]
    // #[case::simple_lte("<=", vec![Token::LteOp])]
    // #[case::simple_gte(">", vec![Token::GtOp])]
    // #[case::simple_lte("<", vec![Token::LtOp])]
    // #[case::simple_lte("+", vec![Token::PlusOp])]
    // #[case::simple_lte("-", vec![Token::MinusOp])]
    // #[case::simple_lte("*", vec![Token::MultOp])]
    // #[case::simple_lte("/", vec![Token::DivOp])]
    // #[case::simple_lte("(", vec![Token::OParen])]
    // #[case::simple_lte(")", vec![Token::CParen])]
    // #[case::one_lead_space(" 1", vec![Token::Int(1)])]
    // #[case::mult_lead_space("    1", vec![Token::Int(1)])]
    fn tokenize_one(#[case] text: &str, #[case] exp: Vec<Token>) {
        let tokens: Result<Vec<Token>, TokenErr> = tokenize(text).collect();
        let err_msg =
            format!("input '{text}' should tokenize successfully, instead got {tokens:?}");
        let act = tokens.expect(&err_msg);

        assert_eq!(act, exp)
    }

    // #[test]
    // fn tokenize_many() {
    //     let input = "25 == ( 10 * 20 - 100) / 4 > 20";
    //     let exp = vec![
    //         Token::Int(25),
    //         Token::EqOp,
    //         Token::OParen,
    //         Token::Int(10),
    //         Token::MultOp,
    //         Token::Int(20),
    //         Token::MinusOp,
    //         Token::Int(100),
    //         Token::CParen,
    //         Token::DivOp,
    //         Token::Int(4),
    //         Token::GtOp,
    //         Token::Int(20),
    //     ];

    //     let tokens: Result<Vec<Token>, TokenErr> = tokenize(input).collect();
    //     let err_msg =
    //         format!("input '{input}' should tokenize successfully, instead got {tokens:?}");
    //     let act = tokens.expect(&err_msg);

    //     assert_eq!(act, exp)
    // }
}
