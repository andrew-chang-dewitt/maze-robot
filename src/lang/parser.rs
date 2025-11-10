//! ## Parser
//!
//! starting with a simple test grammar to make sure building a shift reduce parser works & we can
//! test it effectively:
//!
//! ```ignore
//! expression <e>  ::= <c> | <c> "==" <e>
//! comparison <c>  ::= <t> | <t> <co> <c>
//! comp ops   <co> ::= ">=" | "<=" | ">" | "<"
//! term       <t>  ::= <f> | <f> <to> <t>
//! term ops   <to> ::= "+" | "-"
//! factor     <f>  ::= <p> | <p> <fo> <f>
//! factor ops <fo> ::= "*" | "/"
//! primatives <p>  ::= INTEGER | "(" <e> ")"
//! ```
//!
//! eventually will convert to full grammar, defined as:
//!
//! ```ignore
//! program          ::= <s> EOF
//!
//! statements  <s>  ::= "let" <id> "=" <e>
//!                    | "def" <id>
//!                    | "if" <e> "then" <s> "else" <s>
//!                    | "while" <e> "do" <s>
//!                    | <s>";" <s>
//!                    | SKIP
//!
//! expressions <e>  ::= <qe> | "[" <lm> "]"
//! list mems   <lm> ::= <qe> "," <lm> | <qe> "," | <qe>
//! equality    <qe> ::= <ce> | <ce> <eo> <ee>
//! eq ops      <qo> ::= "==" |"!="
//! comparison  <ce> ::= <te> | <te> <co> <ce>
//! comp ops    <co> ::= ">=" | "<=" | ">" | "<"
//! term        <te> ::= <fe> | <fe> <to> <te>
//! term ops    <to> ::= "+" | "-"
//! factor      <fe> ::= <pe> | <pe> <fo> <fe>
//! factor ops  <fo> ::= "*" | "/"
//! primatives  <pe>  ::= <i> | "true" | "false" | EMPTY | "(" <e> ")"
//! integer     <i>  ::= <ni> | <pi>
//! negative    <ni> ::= "-" <nc> <pi>
//! positive    <pi> ::= <nc> <pi> | EMPTY
//!
//! identifier  <id> ::= <ac> | <ac> <cs>
//! characters  <cs> ::= <ac> <cs> | <nc> <cs> | <sc> <cs> | EMPTY
//! alphabetic  <ac> ::= [a-zA-Z]
//! numeric     <nc> ::= [0-9]
//! special     <sc> ::= [_`~!@#$%^&*\-+\\':;<>,.?/]
//! ```
//!
//! other components will be included in the standard library, such as:
//!
//! - logic functions: `not`, `and`, `or`
//! - list functions: `push_front`, `pop_front`, `push_rear`, `pop_rear`, `is_empty`, `in`, `for_each`, & `map`
//! - hashmap data structure & functions: `create_map` | `get` | `set` | `keys` | `values` | `pairs`
//! - direction enumeration as constants: `UP`, `RIGHT`, `DOWN`, & `LEFT`
//! - robot functions: `move` & `peek`
use std::{error::Error, fmt::Display};

pub fn parse(text: &str) -> Result<ParseTree, ParseErr> {
    todo!()
}

#[derive(Debug)]
pub struct ParseTree(Box<Expression>);

impl PartialEq for ParseTree {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}
impl Eq for ParseTree {}

impl From<Expression> for ParseTree {
    fn from(value: Expression) -> Self {
        Self(Box::new(value))
    }
}

#[derive(Debug)]
pub struct ParseErr(String);

impl Error for ParseErr {}

impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseErr: {}", self.0)
    }
}

#[derive(Debug)]
pub enum Expression {
    Eq(EqExpr),
    Comp(CompExpr),
    Term(TermExpr),
    Fact(FactExpr),
    Prim(PrimExpr),
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Eq(s), Self::Eq(o)) => s == o,
            (Self::Comp(s), Self::Comp(o)) => s == o,
            (Self::Term(s), Self::Term(o)) => s == o,
            (Self::Fact(s), Self::Fact(o)) => s == o,
            (Self::Prim(s), Self::Prim(o)) => s == o,
            _ => false,
        }
    }
}
impl Eq for Expression {}

impl From<isize> for Expression {
    fn from(value: isize) -> Self {
        Self::Prim(PrimExpr::from(value))
    }
}

#[derive(Debug)]
pub enum EqExpr {
    Comp(CompExpr),
    Bin(CompExpr, EqOp, Box<EqExpr>),
}

#[derive(Debug)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl PartialEq for EqExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Comp(this), Self::Comp(other)) => todo!(),
            (Self::Bin(this_lhs, this_op, this_rhs), Self::Bin(oth_lhs, oth_op, oth_rhs)) => {
                todo!()
            }
            _ => false,
        }
    }
}
impl Eq for EqExpr {}

/// comparison <c>  ::= <t> | <t> <co> <c>
#[derive(Debug)]
pub enum CompExpr {
    Term(TermExpr),
    Bin(TermExpr, CompOp, Box<CompExpr>),
}

/// comp ops   <co> ::= ">=" | "<=" | ">" | "<"
#[derive(Debug)]
pub enum CompOp {
    Gte,
    Lte,
    Gt,
    Lt,
}

impl PartialEq for CompExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Term(this), Self::Term(other)) => todo!(),
            (Self::Bin(this_lhs, this_op, this_rhs), Self::Bin(oth_lhs, oth_op, oth_rhs)) => {
                todo!()
            }
            _ => false,
        }
    }
}
impl Eq for CompExpr {}

/// term       <t>  ::= <f> | <f> <to> <t>
#[derive(Debug)]
pub enum TermExpr {
    Fact(FactExpr),
    Bin(FactExpr, TermOp, Box<TermExpr>),
}

/// term ops   <to> ::= "+" | "-"
#[derive(Debug)]
pub enum TermOp {
    Plus,
    Minus,
}

impl PartialEq for TermExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Fact(this), Self::Fact(other)) => todo!(),
            (Self::Bin(this_lhs, this_op, this_rhs), Self::Bin(oth_lhs, oth_op, oth_rhs)) => {
                todo!()
            }
            _ => false,
        }
    }
}
impl Eq for TermExpr {}

/// factor     <f>  ::= <p> | <p> <fo> <f>
#[derive(Debug)]
pub enum FactExpr {
    Prim(PrimExpr),
    Bin(PrimExpr, FactOp, Box<FactExpr>),
}

/// factor ops <fo> ::= "*" | "/"
#[derive(Debug)]
pub enum FactOp {
    Mult,
    Div,
}

impl PartialEq for FactExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Prim(this), Self::Prim(other)) => todo!(),
            (Self::Bin(this_lhs, this_op, this_rhs), Self::Bin(oth_lhs, oth_op, oth_rhs)) => {
                todo!()
            }
            _ => false,
        }
    }
}
impl Eq for FactExpr {}

/// primatives <p>  ::= INTEGER | "(" <e> ")"
#[derive(Debug)]
pub struct PrimExpr(isize);

impl From<isize> for PrimExpr {
    fn from(value: isize) -> Self {
        Self(value)
    }
}

impl PartialEq for PrimExpr {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for PrimExpr {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    // #[rstest]
    // #[case("1", ParseTree::from(Expression::from(1)))]
    // #[case("987654321", ParseTree::from(Expression::from(987654321)))]
    // fn parse_primatives(#[case] text: &str, #[case] exp: ParseTree) {
    //     let act = parse(text).expect("text {text} should parse successfully");

    //     assert_eq!(act, exp)
    // }
}
