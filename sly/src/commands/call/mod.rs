use crate::lib::command::{AsyncCommand, Command};
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

mod helper;
mod query;
mod update;

#[derive(Clap)]
pub enum CallSubCommands {
    /// Send a query call
    Query(query::QueryOpts),
    /// Send a query call
    Update(update::UpdateOpts),
}

impl Command for CallSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            CallSubCommands::Query(opts) => opts.exec(env),
            CallSubCommands::Update(opts) => opts.exec(env),
        }
    }
}
