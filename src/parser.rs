use crate::language::{AExpr, BExpr, Command, Program};
use chumsky::prelude::*;

pub type ParserError<'src> = extra::Err<Rich<'src, char>>;

/// Parse an IMP program from a string. Returns a `Program` AST.
pub fn parse_imp<'src>() -> impl Parser<'src, &'src str, Program<'src>, ParserError<'src>> {
    let cmds = recursive(|cmds| {
        type BinOp<'src> = fn(Box<AExpr<'src>>, Box<AExpr<'src>>) -> BExpr<'src>;
        type ArithmeticOp<'src> = fn(Box<AExpr<'src>>, Box<AExpr<'src>>) -> AExpr<'src>;

        let space = text::whitespace().at_least(1);
        let int = text::int(10).from_str::<u64>().unwrapped().map(AExpr::Int);
        const RESERVED_KEYWORDS: &[&str] = &["if", "then", "else", "fi", "skip"];
        let identifier = text::ident()
            .filter(|s: &&str| !RESERVED_KEYWORDS.contains(s))
            .labelled("identifier");

        let a_expr = recursive(|a_expr| {
            let arithmetic_operator = choice((
                just("+").to(AExpr::Add as ArithmeticOp<'src>),
                just("-").to(AExpr::Subtract as ArithmeticOp<'src>),
                just("*").to(AExpr::Multiply as ArithmeticOp<'src>),
            ));
            let atom = choice((
                int,
                identifier.map(AExpr::Variable),
                a_expr.delimited_by(just("("), just(")")),
            ));
            atom.clone().foldl(
                arithmetic_operator.padded().then(atom).repeated(),
                |left, (operator, right)| operator(Box::new(left), Box::new(right)),
            )
        });

        let arithmetic_comparison_op = choice((
            just(">").to(BExpr::Gt as BinOp<'src>),
            just("<").to(BExpr::Lt as BinOp<'src>),
            just("<>").to(BExpr::Ne as BinOp<'src>),
            just("==").to(BExpr::Eq as BinOp<'src>),
        ))
        .padded();
        let arithmetic_comparison = a_expr
            .clone()
            .then(arithmetic_comparison_op)
            .then(a_expr.clone())
            .map(|((left, operator), right)| operator(Box::new(left), Box::new(right)));
        let bool_expr = recursive(|bool_expr| {
            let atom = choice((
                just("true").to(BExpr::True),
                just("false").to(BExpr::False),
                just("not")
                    .ignore_then(space)
                    .ignore_then(bool_expr.clone())
                    .map(|expr| BExpr::Not(Box::new(expr))),
                arithmetic_comparison,
                bool_expr.delimited_by(just("("), just(")")),
            ));

            #[derive(Clone)]
            enum BoolOp {
                And,
                Or,
            }
            let bool_op = choice((
                text::ascii::keyword("and").to(BoolOp::And),
                text::ascii::keyword("or").to(BoolOp::Or),
            ));
            let tail = space
                .ignore_then(bool_op)
                .then_ignore(space)
                .then(atom.clone())
                .repeated();

            // This avoids left-recursion issues
            atom.foldl(tail, |left, (op, right)| match op {
                BoolOp::And => BExpr::And(Box::new(left), Box::new(right)),
                BoolOp::Or => BExpr::Or(Box::new(left), Box::new(right)),
            })
        });

        let declaration = identifier
            .then_ignore(just(":=").padded())
            .then(a_expr)
            .map(|(name, value)| Command::Assignment {
                name,
                value: Box::new(value),
            });

        let if_statement = just("if")
            .ignore_then(space)
            .ignore_then(bool_expr.clone())
            .then_ignore(space)
            .then_ignore(text::ascii::keyword("then"))
            .then(cmds.clone())
            .then_ignore(text::ascii::keyword("else"))
            .then(cmds.clone())
            .then_ignore(text::ascii::keyword("fi"))
            .map(|((condition, then_cmds), else_cmds)| Command::If {
                condition: Box::new(condition),
                then_cmds,
                else_cmds,
            });

        let while_statement = just("while")
            .ignore_then(space)
            .ignore_then(bool_expr.clone())
            .then_ignore(space)
            .then_ignore(text::ascii::keyword("do"))
            .then(cmds.clone())
            .then_ignore(text::ascii::keyword("od"))
            .map(|(condition, cmds)| Command::While {
                condition: Box::new(condition),
                cmds,
            });

        let skip = text::ascii::keyword("skip")
            .to(Command::Skip())
            .labelled("'skip'");
        let command = choice((
            //
            skip,
            declaration,
            if_statement,
            while_statement,
        ));

        command
            .separated_by(just(";").padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .padded()
    });

    cmds.map(Program::Commands)
}
