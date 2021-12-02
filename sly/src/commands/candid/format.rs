use anyhow::bail;
use anyhow::Result;
use clap::Parser as Clap;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use crate::lib::candid::CandidParser;
use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct CandidFormatOpts {
    /// Path to the candid files to format.
    files: Vec<String>,
}

impl Command for CandidFormatOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        let mut parser = CandidParser::default();

        for file in self.files {
            if let Err(diagnostic) = parser.parse(file.as_str()) {
                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();
                term::emit(&mut writer.lock(), &config, &parser, &diagnostic)?;
                bail!("Candid format failed due to parse error.");
            }
        }

        parser.format_all();

        Ok(())
    }
}
