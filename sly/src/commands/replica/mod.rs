//! Manage the local instance of the replica.

use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::{AsyncCommand, Command};
use crate::lib::env::Env;

mod start;
mod status;

#[derive(Clap)]
pub enum ReplicaSubCommands {
    /// Start the local instance of the replica.
    Start(start::ReplicaStartOpts),
    /// Checks the `status` endpoints of the replica
    Status(status::ReplicaStatusOpts),
}

impl Command for ReplicaSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            ReplicaSubCommands::Start(opts) => opts.exec(env),
            ReplicaSubCommands::Status(opts) => opts.exec(env),
        }
    }
}
