use candid::error::Error;
use candid::parser::token;
use candid::{check_prog, types::Type, IDLProg, TypeEnv};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use lalrpop_util::ParseError;
use std::fs;
use std::path::PathBuf;

/// An error produced by the Candid parser.
type ParserError = ParseError<usize, token::Token, token::LexicalError>;

pub struct CandidError {
    pub files: SimpleFiles<String, String>,
    pub diagnostic: Diagnostic<()>,
}

/// Try to parse a candid file and return the `IDLProg`.
pub fn parse(file: &str) -> Result<IDLProg, CandidError> {
    let path = PathBuf::from(file);
    let source = fs::read_to_string(path).expect("Cannot read file.");

    let mut files = SimpleFiles::new();
    let file_id = files.add(file.to_string(), source);
    debug_assert_eq!(file_id, 0);

    file.parse().map_err(|error| CandidError {
        files,
        diagnostic: error_to_diagnostic(error),
    })
}

fn error_to_diagnostic(error: candid::Error) -> Diagnostic<()> {
    match error {
        Error::Parse(error) => {
            let mut dia = Diagnostic::error().with_message("Parse error");

            let label = match error {
                ParserError::InvalidToken { location } => {
                    Label::primary((), location..location + 1).with_message("Invalid Token.")
                }
                ParserError::UnrecognizedEOF { location, expected } => {
                    if expected.len() > 0 {
                        dia = dia.with_notes(vec![expected_text(expected)]);
                    }
                    Label::primary((), location..location + 1)
                        .with_message("Unrecognized End of File.")
                }
                ParserError::UnrecognizedToken { token, expected } => {
                    if expected.len() > 0 {
                        dia = dia.with_notes(vec![expected_text(expected)]);
                    }
                    Label::primary((), token.0..token.2).with_message("Unrecognized Token.")
                }
                ParserError::ExtraToken { token } => {
                    dia = dia.with_notes(vec!["Additional unexpected token found.".into()]);
                    Label::primary((), token.0..token.2).with_message("Extra Token.")
                }
                ParserError::User { error } => Label::primary((), error.span.start..error.span.end)
                    .with_message(format!("Error: {}", error.err)),
            };

            dia = dia.with_labels(vec![label]);
            dia
        }
        Error::Binread(_) => unreachable!(),
        Error::Custom(_) => unreachable!(),
    }
}

fn expected_text<I: Into<String>>(mut expected: Vec<I>) -> String {
    match expected.len() {
        0 => "".into(),
        1 => {
            let expected: String = expected.pop().unwrap().into();
            format!("Expected {}", expected)
        }
        _ => {
            let expected: Vec<String> = expected.into_iter().map(|x| x.into()).collect();
            format!("Expected one of {}", expected.join(", "))
        }
    }
}
