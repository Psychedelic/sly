use crate::lib::candid::CandidParser;
use anyhow::bail;
use anyhow::Result;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub fn run(files: &Vec<String>) -> Result<()> {
    let mut parser = CandidParser::default();

    for file in files {
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
