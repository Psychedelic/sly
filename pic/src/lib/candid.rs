use candid::parser::token::Span;
use candid::parser::types::{Binding, Dec, FuncMode, IDLType, PrimType, ToDoc, TypeField};
use candid::parser::value::IDLField;
use candid::types::{Field, Function, Type};
use candid::{check_file, IDLProg, TypeEnv};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::{Error, Files, SimpleFile, SimpleFiles};
use pathdiff::diff_paths;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

// TODO(qti3e) Move this file to Psychedelic/candid repository.

/// A candid parser that supports includes and has proper error handling.
pub struct CandidParser {
    /// The files that are already visited to guard against infinite circular references.
    visited: BTreeSet<PathBuf>,
    /// The files loaded during the parsing.
    files: SimpleFiles<String, String>,
    /// The IDLProg for each file, sorted by the import order.
    programs: Vec<IDLProg>,
    /// The position for defined types.
    binding_positions: BTreeMap<String, (usize, Span)>,
    /// The type env.
    env: TypeEnv,
}

impl Default for CandidParser {
    fn default() -> Self {
        Self {
            visited: BTreeSet::new(),
            files: SimpleFiles::new(),
            programs: Vec::new(),
            binding_positions: BTreeMap::new(),
            env: TypeEnv::new(),
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

        // Load all of the defined types in all of the candid files.
        for (file_id, prog) in self.programs.iter().enumerate().rev() {
            for dec in &prog.decs {
                match dec {
                    Dec::TypD(binding) => {
                        if self.env.0.contains_key(&binding.id.name) {
                            let pos = self
                                .binding_positions
                                .get(&binding.id.name)
                                .unwrap()
                                .clone();
                            return Err(Diagnostic::error()
                                .with_message("Duplicate name.")
                                .with_labels(vec![
                                    Label::primary(file_id, binding.id.span.clone()),
                                    Label::secondary(pos.0, pos.1)
                                        .with_message("Another definition was found here."),
                                ]));
                        }

                        let span = binding.id.span.clone();

                        let ty = self.check_type(file_id, &binding.typ).map_err(|d| {
                            let label = Label::secondary(file_id, span.clone())
                                .with_message("Error originated from this binding.");
                            d.with_labels(vec![label])
                        })?;

                        let name = &binding.id.name;
                        self.env.0.insert(name.clone(), ty);
                        self.binding_positions.insert(name.clone(), (file_id, span));
                    }
                    Dec::ImportD(_, _) => {}
                }
            }
        }

        // TODO(qti3e) Type check actor service.

        todo!()
    }

    fn check_type(&self, file_id: usize, ty: &IDLType) -> Result<Type, Diagnostic<usize>> {
        match ty {
            IDLType::PrimT(p) => Ok(check_prim(p)),
            IDLType::VarT(id) => match self.env.0.get(&id.name) {
                Some(_) => Ok(Type::Var(id.name.clone())),
                None => Err(Diagnostic::error()
                    .with_message(format!("Unbound type identifier: {}", id.name))
                    .with_labels(vec![Label::primary(file_id, id.span.clone())])),
            },
            IDLType::OptT(t) => {
                let t = self.check_type(file_id, t)?;
                Ok(Type::Opt(Box::new(t)))
            }
            IDLType::VecT(t) => {
                let t = self.check_type(file_id, t)?;
                Ok(Type::Vec(Box::new(t)))
            }
            IDLType::RecordT(fields) => {
                let fs = self.check_fields(file_id, fields)?;
                Ok(Type::Record(fs))
            }
            IDLType::VariantT(fields) => {
                let fs = self.check_fields(file_id, fields)?;
                Ok(Type::Variant(fs))
            }
            IDLType::PrincipalT => Ok(Type::Principal),
            IDLType::FuncT(func) => {
                let mut args = Vec::new();
                for t in func.args.iter() {
                    args.push(self.check_type(file_id, t)?);
                }

                let mut rets = Vec::new();
                for t in func.rets.iter() {
                    rets.push(self.check_type(file_id, t)?);
                }

                if func.modes.len() > 1 {
                    return Err(Diagnostic::error().with_message("Cannot have more than one mode"));
                }

                if func.modes.len() == 1 && func.modes[0] == FuncMode::Oneway && !rets.is_empty() {
                    return Err(Diagnostic::error()
                        .with_message("Oneway function has non-unit return type"));
                }

                let f = Function {
                    modes: func.modes.clone(),
                    args,
                    rets,
                };

                Ok(Type::Func(f))
            }
            IDLType::ServT(bindings) => {
                let ms = self.check_methods(file_id, bindings)?;
                Ok(Type::Service(ms))
            }
            IDLType::ClassT(_, _) => {
                Err(Diagnostic::error().with_message("Service constructor not supported"))
            }
        }
    }

    fn check_fields(
        &self,
        file_id: usize,
        fields: &[TypeField],
    ) -> Result<Vec<Field>, Diagnostic<usize>> {
        // Field label duplication is checked in the parser.
        let mut res = Vec::new();

        for f in fields {
            let ty = self.check_type(file_id, &f.typ)?;
            res.push(Field {
                id: f.label.clone(),
                ty,
            });
        }

        Ok(res)
    }

    fn check_methods(
        &self,
        file_id: usize,
        bindings: &[Binding],
    ) -> Result<Vec<(String, Type)>, Diagnostic<usize>> {
        let mut res = Vec::new();

        for meth in bindings.iter() {
            let t = self.check_type(file_id, &meth.typ)?;

            self.resolve_type_as_func(&t).map_err(|d| {
                let span = meth.id.span.clone();
                let label = Label::primary(file_id, span)
                    .with_message(format!("Method {} has a non-function type.", meth.id.name,));
                d.with_labels(vec![label])
            })?;

            res.push((meth.id.name.to_owned(), t));
        }

        Ok(res)
    }

    /// Resolve a type as a function, this method also guards against circular references.
    fn resolve_type_as_func<'a>(
        &'a self,
        mut ty: &'a Type,
    ) -> Result<&'a Function, Diagnostic<usize>> {
        let mut visited = BTreeSet::new();

        loop {
            match ty {
                Type::Var(name) => {
                    if !visited.insert(name.clone()) {
                        // We've seen the type before, so we were able to resolve it, so it exists
                        // so we can unwrap here.
                        let (file_id, span) = self.binding_positions.get(name).unwrap().clone();
                        let label = Label::primary(file_id, span).with_message("Circular type.");
                        return Err(Diagnostic::error()
                            .with_message(format!("Type {} has circular definition.", name))
                            .with_labels(vec![label]));
                    }

                    ty = match self.env.0.get(name) {
                        None => {
                            return Err(Diagnostic::error()
                                .with_message(format!("Unbound type identifier: {}", name)))
                        }
                        Some(ty) => ty,
                    };
                }
                Type::Func(f) => return Ok(f),
                typ => {
                    return Err(Diagnostic::error()
                        .with_message(format!("Not a function type: Type={}", typ)))
                }
            }
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
                    Label::secondary(file_id, range).with_message("Error originated from import.");
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
        Error::Binread(_) => {
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
