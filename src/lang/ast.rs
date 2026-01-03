use crate::util::List;

#[derive(Debug, PartialEq)]
pub struct Stmt {
    id: String,
    args: List<String>,
    body: Box<Expr>,
}

impl Stmt {
    pub fn new(id: String, args: List<String>, body: Box<Expr>) -> Self {
        Self { id, args, body }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    List(List<Box<Expr>>),
    Fun(String, Box<Expr>),
    Ref(String),
    Num(i32),
}
