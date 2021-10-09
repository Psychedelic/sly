//! Manage the local instance of the replica.

use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

pub mod start;

#[derive(Clap)]
pub enum ReplicaSubCommands {
    /// Start the local instance of the replica.
    Start(start::ReplicaStartOpts),
}

impl Command for ReplicaSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            ReplicaSubCommands::Start(opts) => opts.exec(env),
        }
    }
}
