use crate::lib::candid::CandidParser;
use crate::lib::utils::result_flatten;
use anyhow::{bail, Result};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone)]
pub struct Languages {
    pub typescript: bool,
    pub javascript: bool,
    pub motoko: bool,
}

pub fn run(files: &Vec<String>, dir: &str, languages: Languages) -> Result<()> {
    let dir_path = PathBuf::from(dir);

    mkdirp::mkdirp(dir_path.clone())?;

    for file in files {
        gen_file_bindings(file, dir_path.as_path(), languages)?;
    }

    Ok(())
}

fn gen_file_bindings(file: &str, dir: &Path, languages: Languages) -> Result<()> {
    let mut parser = CandidParser::default();
    let maybe_env = result_flatten(parser.parse(file).map(|_| parser.construct_type_env()));

    if let Err(diagnostic) = maybe_env {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        term::emit(&mut writer.lock(), &config, &parser, &diagnostic)?;
        bail!("Code gen stopped due to parse error.");
    }

    let file_path = PathBuf::from(file);
    let filename = file_path.file_name().unwrap().to_str().unwrap();

    let env = parser.get_type_env();
    let actor = parser.get_service_for(file);

    if languages.javascript {
        let source = candid::bindings::javascript::compile(env, actor);
        let path = dir.join(format!("{}.js", filename));
        fs::write(path.clone(), source).expect(&format!("Write to {:?} failed.", path));
    }

    if languages.typescript {
        let source = candid::bindings::typescript::compile(env, actor);
        let path = dir.join(format!("{}.ts", filename));
        fs::write(path.clone(), source).expect(&format!("Write to {:?} failed.", path));
    }

    if languages.motoko {
        let source = candid::bindings::motoko::compile(env, actor);
        let path = dir.join(format!("{}.mo", filename));
        fs::write(path.clone(), source).expect(&format!("Write to {:?} failed.", path));
    }

    Ok(())
}
