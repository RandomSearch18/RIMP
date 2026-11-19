use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    extra::{Err, ParserExtra},
    prelude::*,
    span::Span,
};

#[derive(Debug)]
enum Program<'src> {
    Commands(Vec<Command<'src>>),
}

#[derive(Debug, Clone)]
enum AExpr<'src> {
    Int(u64),
    Variable(&'src str),
    Add(Box<AExpr<'src>>, Box<AExpr<'src>>),
    Subtract(Box<AExpr<'src>>, Box<AExpr<'src>>),
    Multiply(Box<AExpr<'src>>, Box<AExpr<'src>>),
}

#[derive(Debug, Clone)]
enum BExpr<'src> {
    /// The `true` value
    True,
    /// The `false` value
    False,
    /// Boolean not: `not x`
    Not(Box<BExpr<'src>>),
    /// Boolean and: `x and y`
    And(Box<BExpr<'src>>, Box<BExpr<'src>>),
    /// Boolean or: `x or y`
    Or(Box<BExpr<'src>>, Box<BExpr<'src>>),
    /// Greater than: `x > y`
    Gt(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Less than: `x < y`
    Lt(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Equal to: `x == y`
    Eq(Box<AExpr<'src>>, Box<AExpr<'src>>),
    /// Not equal: `x <> y`
    Ne(Box<AExpr<'src>>, Box<AExpr<'src>>),
}

#[derive(Debug, Clone)]
enum Command<'src> {
    Skip(),
    Assignment {
        name: &'src str,
        value: Box<AExpr<'src>>,
    },
    If {
        condition: Box<BExpr<'src>>,
        then_cmds: Vec<Command<'src>>,
        else_cmds: Vec<Command<'src>>,
    },
    While {
        condition: Box<BExpr<'src>>,
        cmds: Vec<Command<'src>>,
    },
}

type ParserError<'src> = extra::Err<Rich<'src, char>>;

fn parser<'src>() -> impl Parser<'src, &'src str, Program<'src>, ParserError<'src>> {
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

            let arithmetic_operation = a_expr
                .clone()
                .then(arithmetic_operator.padded())
                .then(a_expr.clone())
                .map(|((left, operator), right)| operator(Box::new(left), Box::new(right)));

            choice((
                int,
                identifier.map(AExpr::Variable),
                arithmetic_operation,
                a_expr.delimited_by(just("("), just(")")),
            ))
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

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .expect("Please provide an input file");
    let src = std::fs::read_to_string(&input_file).expect("Failed to read input file");
    let result = parser().parse(&src);
    println!("{:#?}", result);
    if let Err(errors) = result.into_result() {
        for error in errors {
            let span = error.span();
            let range = span.start..span.end;
            Report::build(ReportKind::Error, (input_file.as_str(), range.clone()))
                .with_message("parse error")
                .with_label(
                    Label::new((input_file.as_str(), range))
                        .with_message(error.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint((input_file.as_str(), Source::from(src.as_str())))
                .unwrap();
        }
    }
}
