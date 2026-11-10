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
}

#[derive(Debug, Clone)]
enum Command<'src> {
    Skip(),
    Assignment {
        name: &'src str,
        value: Box<AExpr<'src>>,
    },
}

type ParserError<'src> = extra::Err<Rich<'src, char>>;

fn parser<'src>() -> impl Parser<'src, &'src str, Program<'src>, ParserError<'src>> {
    let command = text::ascii::keyword("skip").to(Command::Skip());

    let cmds = command
        .separated_by(just(";").padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .padded();

    cmds.map(Program::Commands)
}

fn main() {
    let input_file = std::env::args().nth(1).unwrap();
    let src = std::fs::read_to_string(&input_file).unwrap();
    let result = parser().parse(&src);
    println!("{:?}", result);
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
