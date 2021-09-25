use candid::parser::token::Span;
use candid::parser::types::Dec;
use candid::{check_file, IDLProg, TypeEnv};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Error, Files, SimpleFile, SimpleFiles};
use pathdiff::diff_paths;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use thiserror::private::PathAsDisplay;

/// A candid parser that supports includes and has proper error handling.
pub struct CandidParser {
    /// The files that are already visited to guard against infinite circular references.
    visited: BTreeSet<PathBuf>,
    /// The files loaded during the parsing.
    files: SimpleFiles<String, String>,
    /// The IDLProg for each file.
    programs: BTreeMap<usize, IDLProg>,
    /// The imported files in order.
    imports: Vec<usize>,
    /// The collected TypeEnv for the entry file.
    types: Option<TypeEnv>,
}

impl Default for CandidParser {
    fn default() -> Self {
        Self {
            visited: Default::default(),
            files: SimpleFiles::new(),
            programs: Default::default(),
            imports: vec![],
            types: None,
        }
    }
}

impl CandidParser {
    /// Try to parse a file.
    pub fn parse(&mut self, file: &str) -> Result<(), Diagnostic<usize>> {
        let cwd = std::env::current_dir().expect("Cannot get cwd.");
        let path = resolve_path(cwd.as_path(), file);
        self.parse_file_recursive(path)
    }

    fn parse_file_recursive(&mut self, path: PathBuf) -> Result<(), Diagnostic<usize>> {
        if self.visited.contains(&path) {
            // The file is already loaded so we don't need to load it again.
            return Ok(());
        }

        let display_name = display_path(&path);

        let source = fs::read_to_string(&path).map_err(|e| {
            Diagnostic::error().with_message(format!("Cannot read file '{}': {}", display_name, e))
        })?;

        let file_id = self.files.add(display_name, source);
        let source = self.files.get(file_id).unwrap().source();

        let program: IDLProg = source
            .parse()
            .map_err(|error| candid_error_to_diagnostic(file_id, error))?;

        let mut imports: Vec<(PathBuf, Span)> = Vec::new();

        // Now resolve the imports.
        let base = path.parent().unwrap();
        for dec in &program.decs {
            if let Dec::ImportD(file, loc) = dec {
                imports.push((resolve_path(base, file.as_str()), loc.clone()));
            }
        }

        self.programs.insert(file_id, program);
        self.imports.push(file_id);

        for (path, range) in imports {
            self.parse_file_recursive(path).map_err(|d| {
                if d.labels.is_empty() {
                    d.with_labels(vec![Label::primary(file_id, range)])
                } else {
                    d
                }
            })?;
        }

        Ok(())
    }
}

impl<'a> Files<'a> for CandidParser {
    type FileId = usize;
    type Name = String;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        self.files.name(id)
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        self.files.source(id)
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.files.line_index(id, byte_index)
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.files.line_range(id, line_index)
    }
}

/// Join two paths together.
fn resolve_path(base: &Path, file: &str) -> PathBuf {
    let file = shellexpand::tilde(file);
    let file = PathBuf::from(file.as_ref());
    if file.is_absolute() {
        file
    } else {
        base.join(file)
    }
}

/// Compute the relative path to the given path from the cwd and return the result as an
/// string.
fn display_path(path: &PathBuf) -> String {
    let cwd = std::env::current_dir().expect("Cannot get cwd.");
    let relative = diff_paths(path, cwd).unwrap();
    relative.to_str().unwrap().to_string()
}

/// Convert an error produced by the candid parser to a diagnostic on the given file.
fn candid_error_to_diagnostic(file_id: usize, error: candid::error::Error) -> Diagnostic<usize> {
    use candid::error::Error;

    match error {
        Error::Parse(e) => {
            use lalrpop_util::ParseError::*;
            let mut diag = Diagnostic::error().with_message("Parser error");
            let label = match e {
                User { error } => {
                    Label::primary(file_id, error.span.clone()).with_message(&error.err)
                }
                InvalidToken { location } => {
                    Label::primary(file_id, location..location + 1).with_message("Invalid token")
                }
                UnrecognizedEOF { location, expected } => {
                    diag = diag.with_notes(report_expected(&expected));
                    Label::primary(file_id, location..location + 1).with_message("Unexpected EOF")
                }
                UnrecognizedToken { token, expected } => {
                    diag = diag.with_notes(report_expected(&expected));
                    Label::primary(file_id, token.0..token.2).with_message("Unexpected token")
                }
                ExtraToken { token } => {
                    Label::primary(file_id, token.0..token.2).with_message("Extra token")
                }
            };
            diag.with_labels(vec![label])
        }
        Error::Binread(labels) => {
            unreachable!("Unexpected bin-read error.")
        }
        Error::Custom(e) => Diagnostic::error().with_message(e.to_string()),
    }
}

fn report_expected(expected: &[String]) -> Vec<String> {
    if expected.is_empty() {
        return Vec::new();
    }
    use pretty::RcDoc;
    let doc: RcDoc<()> = RcDoc::intersperse(
        expected.iter().map(RcDoc::text),
        RcDoc::text(",").append(RcDoc::softline()),
    );
    let header = if expected.len() == 1 {
        "Expects"
    } else {
        "Expects one of"
    };
    let doc = RcDoc::text(header).append(RcDoc::softline().append(doc));
    vec![doc.pretty(70).to_string()]
}

#[test]
fn x() {
    let path = Path::new("/Users/qti3e/Code/icx/tmp/B.did");
    let source = fs::read_to_string(&path).unwrap();
    let prog: IDLProg = source.parse().unwrap();

    println!("{:#?}", prog);
    // println!("{:#?}", check_file(&path));
}

#[test]
fn z() {
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

    let mut parser = CandidParser::default();

    if let Err(dia) = parser.parse("/Users/qti3e/Code/icx/tmp/B.did") {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        codespan_reporting::term::emit(&mut writer.lock(), &config, &parser, &dia).unwrap();
    }
}

// #[test]
// fn y() {
//     let mut parser = CandidParser::default();
//     parser.parse("X.did");
//     parser.type_env();
// }
