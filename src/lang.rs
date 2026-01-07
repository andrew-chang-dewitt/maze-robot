mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser, "/lang/grammar.rs");

#[cfg(test)]
mod tests {
    use lalrpop_util::ParseError;
    use nostd::{boxed::Box, prelude::ToString, string::String, vec};
    use rstest::rstest;

    use crate::ds::List;

    use super::ast::{Expr, Prog, Stmt};
    use super::parser::{ExprParser, ProgParser, StmtParser};

    #[test]
    fn prog() {
        let input = "foo = 1\n\nbar x = x";
        let expected = Prog::new(List::cons(
            Stmt::new("foo", List::empty(), Expr::Num(1)),
            List::one(Stmt::new(
                "bar",
                List::one("x".to_string()),
                Expr::Ref("x".to_string()),
            )),
        ));
        let actual = ProgParser::new()
            .parse(input)
            .expect("{input} should parse");

        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::single_arg(
        "foo x = 1",
        "foo",
        List::one("x".to_string()),
        Expr::Num(1)
    )]
    #[case::many_arg(
        "foo x y = 1",
        "foo",
        List::cons("x".to_string(), List::one("y".to_string())),
        Expr::Num(1)
    )]
    #[case::zero_arg("foo = 1", "foo", List::empty(), Expr::Num(1))]
    fn fun_stmt(
        #[case] input: &str,
        #[case] exp_id: &str,
        #[case] exp_args: List<String>,
        #[case] exp_body: Expr,
    ) {
        let actual = StmtParser::new()
            .parse(input)
            .expect("{input} should parse");
        let expected = Stmt::new(exp_id, exp_args, exp_body);

        assert_eq!(actual, Box::new(expected))
    }

    #[rstest]
    #[case::one_digit("1", Expr::Num(1))]
    #[case::many_digits("1234567890", Expr::Num(1234567890))]
    #[case::one_separator("1_2", Expr::Num(12))]
    #[case::many_separators("1_23_____456_", Expr::Num(123456))]
    fn ints_ok(#[case] input: &str, #[case] output: Expr) {
        let actual = ExprParser::new()
            .parse(input)
            .expect("{input} should parse");

        assert_eq!(actual, Box::new(output))
    }

    #[test]
    fn ints_err() {
        let input = "_1";
        let expected_location = 0;
        let actual = ExprParser::new().parse(input);

        match actual {
            Err(ParseError::InvalidToken { location }) => assert_eq!(
                location, expected_location,
                "invalid token should be at {expected_location}"
            ),
            Err(e) => panic!("expected `ParseError::InvalidToken`, got {e}"),
            Ok(x) => panic!("expected parsing {input} to fail, instead got {x:?}"),
        }
    }

    #[rstest]
    #[case::single_pair("(22)", Expr::Num(22))]
    #[case::many_pair("((((22))))", Expr::Num(22))]
    fn parens_ok(#[case] input: &str, #[case] output: Expr) {
        let actual = ExprParser::new()
            .parse(input)
            .expect("{input} should parse");

        assert_eq!(actual, Box::new(output))
    }

    #[test]
    fn parens_mismatched() {
        //                 01234
        let input = "((22)";
        let expected_location = 5;
        let expected_token = vec!["CParen".to_string()];
        let actual = ExprParser::new().parse(input);

        match actual {
            Err(ParseError::UnrecognizedEof { location, expected }) => {
                assert_eq!(
                    (&location, &expected),
                    (&expected_location, &expected_token),
                    "bad end of file should be at {expected_location} & \
                    missing token should be {expected_token:?}"
                )
            }
            Err(e) => panic!("expected `ParseError::UnrecognizedEof`, got {e}"),
            Ok(x) => panic!("expected parsing {input} to fail, instead got {x:?}"),
        }
    }

    #[rstest]
    #[case::int_arg("foo 1", "foo", Expr::Num(1))]
    #[case::expr_arg("foo (1)", "foo", Expr::Num(1))]
    #[case::fn_arg("foo (bar 1)", "foo", Expr::Fun("bar".to_string(), Box::new(Expr::Num(1))))]
    // #[case::no_arg("foo", true)]
    fn fn_expr(#[case] input: &str, #[case] exp_id: String, #[case] exp_body: Expr) {
        let actual = ExprParser::new()
            .parse(input)
            .expect("{input} should parse");

        match *actual {
            Expr::Fun(act_id, act_body) => {
                assert_eq!((act_id, act_body), (exp_id, Box::new(exp_body)))
            }
            _ => panic!("expected an `Expr::Fun`, but got {actual:?}"),
        }
    }

    #[rstest]
    #[case::empty("[]", Expr::List(List::empty()))]
    #[case::one_int("[1]", Expr::List(List::one(Box::new(Expr::Num(1)))))]
    #[case::one_fun("[foo 1]", Expr::List(List::one(Box::new(Expr::Fun("foo".to_string(), Box::new(Expr::Num(1)))))))]
    #[case::trailing_comma("[1,]", Expr::List(List::one(Box::new(Expr::Num(1)))))]
    #[case::many(
        "[1,2]",
        Expr::List(List::cons(Box::new(Expr::Num(1)), List::one(Box::new(Expr::Num(2)))))
    )]
    fn list_expr(#[case] input: &str, #[case] expected: Expr) {
        let actual = ExprParser::new()
            .parse(input)
            .expect("{input} should parse");

        assert_eq!(actual, Box::new(expected))
    }
}
