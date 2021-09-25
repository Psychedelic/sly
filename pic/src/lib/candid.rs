use candid::parser::token::Span;
use candid::parser::types::{Dec, IDLType, PrimType};
use candid::{check_file, IDLProg, TypeEnv};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Error, Files, SimpleFile, SimpleFiles};
use pathdiff::diff_paths;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use candid::types::Type;

// TODO(qti3e) Move this file to Psychedelic/candid repository.

/// A candid parser that supports includes and has proper error handling.
pub struct CandidParser {
    /// The files that are already visited to guard against infinite circular references.
    visited: BTreeSet<PathBuf>,
    /// The files loaded during the parsing.
    files: SimpleFiles<String, String>,
    /// The IDLProg for each file, sorted by the import order.
    programs: Vec<IDLProg>,
    /// The type env.
    env: TypeEnv
}

impl Default for CandidParser {
    fn default() -> Self {
        Self {
            visited: BTreeSet::new(),
            files: SimpleFiles::new(),
            programs: Vec::new(),
            env: TypeEnv::new()
        }
    }
}

impl CandidParser {
    /// Try to parse a file.
    pub fn parse(&mut self, file: &str) -> Result<(), Diagnostic<usize>> {
        let cwd = std::env::current_dir().expect("Cannot get cwd.");
        let path = resolve_path(cwd.as_path(), file);
        let mut visited = BTreeSet::new();
        self.parse_file_recursive(path, &mut visited)
    }

    /// Return the type context for the entire parsed candid files.
    pub fn type_env(&mut self) -> Result<TypeEnv, Diagnostic<usize>> {
        assert!(
            !self.programs.is_empty(),
            "Cannot obtain the type when no file is parsed."
        );

        self.env = TypeEnv::new();

        for (file_id, prog) in self.programs.iter().enumerate().rev() {
            for dec in &prog.decs {
                match dec {
                    Dec::TypD(binding) => {
                        if self.env.0.contains_key(&binding.id.name) {
                            return Err(Diagnostic::error()
                                .with_message("Duplicate name.")
                                .with_labels(vec![Label::primary(
                                    file_id,
                                    binding.id.span.clone(),
                                )]));
                        }
                    }
                    Dec::ImportD(_, _) => {}
                }
            }
        }

        println!("{:?}", self.imports);

        todo!()
    }

    fn check_type(&self, file_id: usize, ty: &IDLType) -> Result<Type, Diagnostic<usize>> {
        match ty {
            IDLType::PrimT(p) => Ok(check_prim(p)),
            IDLType::VarT(id) => {
                match self.env.0.get(&id.name) {
                    Some(ty) => {
                        Ok(Type::Var(id.name.clone()))
                    },
                    None => {
                        Err(Diagnostic::error()
                            .with_message(format!("Unbound type identifier: {}", id.name))
                            .with_labels(vec![
                                Label::primary(file_id, id.span.clone())
                            ])
                        )
                    }
                }
            }
            IDLType::OptT(t) => {
                let t = self.check_type(file_id, t)?;
                Ok(Type::Opt(Box::new(t)))
            }
            IDLType::VecT(t) => {
                let t = self.check_type(file_id, t)?;
                Ok(Type::Vec(Box::new(t)))
            }
            IDLType::RecordT(_) => {}
            IDLType::VariantT(_) => {}
            IDLType::PrincipalT => Ok(Type::Principal),
            IDLType::FuncT(_) => {}
            IDLType::ServT(_) => {}
            IDLType::ClassT(_, _) => {}
        }
    }

    fn parse_file_recursive(
        &mut self,
        path: PathBuf,
        visited: &mut BTreeSet<PathBuf>,
    ) -> Result<(), Diagnostic<usize>> {
        if visited.contains(&path) {
            return Err(Diagnostic::error().with_message("Recursive import."));
        }

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

        self.programs.push(program);

        self.visited.insert(path.clone());
        visited.insert(path.clone());

        for (path, range) in imports {
            self.parse_file_recursive(path, visited).map_err(|d| {
                let label =
                    Label::primary(file_id, range).with_message("Error originated from import.");
                d.with_labels(vec![label])
            })?;
        }

        visited.remove(&path);

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

fn check_prim(prim: &PrimType) -> Type {
    match prim {
        PrimType::Nat => Type::Nat,
        PrimType::Nat8 => Type::Nat8,
        PrimType::Nat16 => Type::Nat16,
        PrimType::Nat32 => Type::Nat32,
        PrimType::Nat64 => Type::Nat64,
        PrimType::Int => Type::Int,
        PrimType::Int8 => Type::Int8,
        PrimType::Int16 => Type::Int16,
        PrimType::Int32 => Type::Int32,
        PrimType::Int64 => Type::Int64,
        PrimType::Float32 => Type::Float32,
        PrimType::Float64 => Type::Float64,
        PrimType::Bool => Type::Bool,
        PrimType::Text => Type::Text,
        PrimType::Null => Type::Null,
        PrimType::Reserved => Type::Reserved,
        PrimType::Empty => Type::Empty,
    }
}
