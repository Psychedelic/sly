use crate::lib::command::{AsyncCommand, Command};
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

mod query;

#[derive(Clap)]
pub enum CallSubCommands {
    /// Send a query call
    Query(query::QueryOpts),
}

impl Command for CallSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            CallSubCommands::Query(opts) => opts.exec(env),
        }
    }
}
