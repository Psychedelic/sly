use crate::lib::candid::CandidParser;
use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::utils::result_flatten;
use anyhow::bail;
use anyhow::Result;
use clap::Clap;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Clap)]
pub struct CandidCheckOpts {
    /// Path to the candid file.
    filename: String,
}

impl Command for CandidCheckOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut parser = CandidParser::default();

        let filename = self.filename.as_str();
        let maybe_env = result_flatten(parser.parse(filename).map(|_| parser.construct_type_env()));

        if let Err(diagnostic) = maybe_env {
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();
            term::emit(&mut writer.lock(), &config, &parser, &diagnostic)?;
            bail!("Candid check failed.");
        }

        Ok(())
    }
}
