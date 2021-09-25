use crate::lib::candid::CandidParser;
use anyhow::bail;
use anyhow::Result;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub fn run(filename: &str) -> Result<()> {
    let mut parser = CandidParser::default();

    if let Err(diagnostic) = parser.parse(filename) {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        term::emit(&mut writer.lock(), &config, &parser, &diagnostic)?;
        bail!("Candid check failed.");
    }

    Ok(())
}
