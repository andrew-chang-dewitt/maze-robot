//! TODO:
//! ---
//!
//! - [x] grammar spec
//! - [ ] as numbered productions
//! - [ ] lexer/tokenizer
//! - [ ] parser
//! - [ ] ast def
//! Implementation of a simple imperative language capable of being expressed spatially, as puzzle
//! pieces, for users to write maze solving programs with.
//!
//! Grammar definition:
//! ---
//!
//! ```
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

/// Test parser
///
/// TODO:
/// ---
///
/// - [x] grammar spec
/// - [ ] as numbered productions
/// - [ ] lexer/tokenizer
/// - [ ] parser
/// - [ ] ast def
///
/// starting with a simple test grammar to make sure building a shift reduce parser works & we can
/// test it effectively:
///
/// ```
/// expression <e>  ::= <c> | <c> "==" <e>
/// comparison <c>  ::= <t> | <t> <co> <c>
/// comp ops   <co> ::= ">=" | "<=" | ">" | "<"
/// term       <t>  ::= <f> | <f> <to> <t>
/// term ops   <to> ::= "+" | "-"
/// factor     <f>  ::= <p> | <p> <fo> <f>
/// factor ops <fo> ::= "*" | "/"
/// primatives <p>  ::= INTEGER | "(" <e> ")"
/// ```

enum Expr {
    Comp(CompExpr),
    Bin(CompExpr, EqOp, Box<Expr>),
}

enum EqOp {
    Equal,
    NotEqual,
}

/// comparison <c>  ::= <t> | <t> <co> <c>
enum CompExpr {
    Term(TermExpr),
    Bin(TermExpr, CompOp, Box<CompExpr>),
}

/// comp ops   <co> ::= ">=" | "<=" | ">" | "<"
enum CompOp {
    Gte,
    Lte,
    Gt,
    Lt,
}

/// term       <t>  ::= <f> | <f> <to> <t>
enum TermExpr {
    Fact(FactExpr),
    Bin(FactExpr, TermOp, Box<TermExpr>),
}

/// term ops   <to> ::= "+" | "-"
enum TermOp {
    Plus,
    Minus,
}

/// factor     <f>  ::= <p> | <p> <fo> <f>
enum FactExpr {
    Prim(isize),
    Bin(isize, FactOp, Box<FactExpr>),
}

/// factor ops <fo> ::= "*" | "/"
enum FactOp {
    Mult,
    Div,
}
