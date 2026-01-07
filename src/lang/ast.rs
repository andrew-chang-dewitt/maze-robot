use nostd::{boxed::Box, prelude::ToString, string::String};

use crate::ds::List;

#[derive(Debug, PartialEq)]
pub struct Prog(List<Stmt>);

impl Prog {
    pub fn new(stmts: List<Stmt>) -> Self {
        Self(stmts)
    }
}

#[derive(Debug, PartialEq)]
pub struct Stmt {
    id: String,
    args: List<String>,
    body: Expr,
}

impl Stmt {
    pub fn new(id: &str, args: List<String>, body: Expr) -> Self {
        Self {
            id: id.to_string(),
            args,
            body,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    List(List<Box<Expr>>),
    Fun(String, Box<Expr>),
    Ref(String),
    Num(usize),
}
