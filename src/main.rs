use chumsky::prelude::*;

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

fn parser<'src>() -> impl Parser<'src, &'src str, Program<'src>> {
    let command = text::ascii::keyword("skip").to(Command::Skip());

    let cmds = command
        .separated_by(just(";").padded())
        .allow_trailing()
        .collect::<Vec<_>>()
        .padded();

    cmds.map(Program::Commands)
}

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    println!("{:?}", parser().parse(&src));
}
