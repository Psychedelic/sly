use codespan_reporting::files::{Files, Error, SimpleFile};
use candid::{check_file, IDLProg, TypeEnv};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::ops::Range;
use std::fmt::{Display, Formatter};
use std::fs;
use candid::parser::types::Dec;

/// A candid parser that supports includes and has proper error handling.
pub struct CandidParser {
    /// The files loaded during the parsing.
    files: BTreeMap<String, CandidFile>,
    /// Absolute path of the imported files, sorted by when we encountered them during
    /// parsing. The entry file is always the first item.
    imports: Vec<String>,
    /// The collected TypeEnv for the entry file.
    types: Option<TypeEnv>
}

struct CandidFile {
    source: SimpleFile<FileNameHack, String>,
    program: Option<IDLProg>,
}

impl CandidParser {
    pub fn from_path(filename: &str) {
    }

    fn load_file_recursive(&mut self, base: &Path, file: &str) {
        let path = resolve_path(base, file);
        let path_as_string = path.to_string_lossy().to_string();

        if self.files.contains_key(&path_as_string) {
            // The file is already loaded don't try to load it again.
            return;
        }

        let source = fs::read_to_string(path);
    }

    fn visit_imports(&self, base: &Path, prog: &IDLProg) {
        for dec in &prog.decs {
            if let Dec::ImportD(file, loc) = dec {
                let path = resolve_path(base, file);
            }
        }
    }

    fn get(&self, id: &String) -> Result<&CandidFile, Error> {
        self.files.get(id).ok_or(Error::FileMissing)
    }
}

impl<'a> Files<'a> for CandidParser {
    type FileId = &'a String;
    type Name = &'a String;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        Ok(self.files.get_key_value(id).ok_or(Error::FileMissing)?.0)
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        Ok(self.get(id)?.source.source())
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.get(id)?.source.line_index((), byte_index)
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.get(id)?.source.line_range((), line_index)
    }
}

/// A new-type type which implements Display so it can be used as Name in SimpleFile.
#[derive(Debug, Copy, Clone)]
struct FileNameHack();

impl Display for FileNameHack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FILE")
    }
}

fn resolve_path(base: &Path, file: &str) -> PathBuf {
    let file = shellexpand::tilde(file);
    let file = PathBuf::from(file.as_ref());
    if file.is_absolute() {
        file
    } else {
        base.join(file)
    }
}

#[test]
fn x() {
    let path = Path::new("/Users/qti3e/Code/icx/tmp/B.did");
    let source = fs::read_to_string(&path).unwrap();
    let prog: IDLProg = source.parse().unwrap();

    println!("{:#?}", prog);
    // println!("{:#?}", check_file(&path));
}