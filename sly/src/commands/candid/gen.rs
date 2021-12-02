use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::Parser as Clap;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use crate::lib::candid::CandidParser;
use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::utils::result_flatten;

#[derive(Clap)]
pub struct CandidGenOpts {
    /// Path to the candid files.
    files: Vec<String>,
    #[clap(short, long)]
    /// Directory
    out_dir: String,
    /// Generate JavaScript language bindings.
    #[clap(long)]
    js: bool,
    /// Generate TypeScript language bindings.
    #[clap(long)]
    ts: bool,
    /// Generate Motoko language bindings.
    #[clap(long)]
    motoko: bool,
}

impl Command for CandidGenOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        let dir = PathBuf::from(&self.out_dir);

        mkdirp::mkdirp(&dir)?;

        for file in &self.files {
            gen_file_bindings(file.as_str(), &dir, &self)?;
        }

        Ok(())
    }
}

fn gen_file_bindings(file: &str, dir: &Path, opts: &CandidGenOpts) -> Result<()> {
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

    if opts.js {
        let source = candid::bindings::javascript::compile(env, actor);
        let path = dir.join(format!("{}.js", filename));
        fs::write(path.clone(), source).unwrap_or_else(|_| panic!("Write to {:?} failed.", path));
    }

    if opts.ts {
        let source = candid::bindings::typescript::compile(env, actor);
        let path = dir.join(format!("{}.ts", filename));
        fs::write(path.clone(), source).unwrap_or_else(|_| panic!("Write to {:?} failed.", path));
    }

    if opts.motoko {
        let source = candid::bindings::motoko::compile(env, actor);
        let path = dir.join(format!("{}.mo", filename));
        fs::write(path.clone(), source).unwrap_or_else(|_| panic!("Write to {:?} failed.", path));
    }

    Ok(())
}
