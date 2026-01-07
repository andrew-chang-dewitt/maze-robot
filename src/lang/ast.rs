use nostd::{boxed::Box, collections::HashMap, prelude::ToString, string::String};

use crate::ds::List;

#[derive(Debug, PartialEq)]
pub struct Prog(HashMap<Sym, Stmt>);

impl Prog {
    pub fn new(stmts: List<Stmt>) -> Self {
        let map: HashMap<Sym, Stmt> = stmts.foldr(HashMap::new(), |mut m, s| {
            m.insert(s.id.clone(), s);
            m
        });

        Self(map)
    }
}

#[derive(Debug, PartialEq)]
pub struct Stmt {
    id: Sym,
    args: List<Sym>,
    body: Expr,
}

impl Stmt {
    pub fn new(id: Sym, args: List<Sym>, body: Expr) -> Self {
        Self { id, args, body }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    List(List<Box<Expr>>),
    Fun(Sym, Box<Expr>),
    Ref(Sym),
    Num(usize),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Sym(pub String);

impl Sym {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}
