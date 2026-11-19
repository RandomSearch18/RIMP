use std::path::Path;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
mod language;
mod parser;
use parser::parse_imp;

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .expect("Please provide an input file");
    let input_file = Path::new(&input_file);
    parse_and_print_errors(input_file);
}

fn parse_and_print_errors(input_file: &Path) {
    let src = std::fs::read_to_string(&input_file).expect("failed to read input file");
    let result = parse_imp().parse(&src);
    println!("{:#?}", result);
    if let Err(errors) = result.into_result() {
        let path_str = input_file.to_str().unwrap_or("<invalid file path>");
        for error in errors {
            let span = error.span();
            let range = span.start..span.end;
            Report::build(ReportKind::Error, (path_str, range.clone()))
                .with_message("parse error")
                .with_label(
                    Label::new((path_str, range))
                        .with_message(error.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint((path_str, Source::from(src.as_str())))
                .unwrap();
        }
    }
}
