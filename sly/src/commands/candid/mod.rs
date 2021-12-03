use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

pub mod check;
pub mod format;
pub mod gen;

#[derive(Clap)]
pub enum CandidSubCommands {
    /// Run the type verifier on a candid file.
    Check(check::CandidCheckOpts),
    /// Format a candid file.
    Format(format::CandidFormatOpts),
    /// Generate codes for the Candid files.
    Gen(gen::CandidGenOpts),
}

impl Command for CandidSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            CandidSubCommands::Check(opts) => opts.exec(env),
            CandidSubCommands::Format(opts) => opts.exec(env),
            CandidSubCommands::Gen(opts) => opts.exec(env),
        }
    }
}
