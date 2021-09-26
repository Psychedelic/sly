use crate::lib::candid::CandidParser;
use crate::lib::utils::result_flatten;
use anyhow::bail;
use anyhow::Result;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub fn run(filename: &str) -> Result<()> {
    let mut parser = CandidParser::default();

    let maybe_env = result_flatten(parser.parse(filename).map(|_| parser.construct_type_env()));

    if let Err(diagnostic) = maybe_env {
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        term::emit(&mut writer.lock(), &config, &parser, &diagnostic)?;
        bail!("Candid check failed.");
    }

    Ok(())
}
