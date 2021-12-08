use anyhow::Result;
use async_trait::async_trait;
use clap::Parser as Clap;

use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

pub mod helper;
pub mod query;
pub mod update;
pub mod waiter;

#[derive(Clap)]
pub struct CallSubCommands {
    /// Path to a candid file to analyze the argument
    #[clap(long, short)]
    candid: Option<String>,
    /// The type of input (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    in_type: helper::ArgType,
    /// The type of output (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    out_type: helper::ArgType,
    /// An optional field to set the expiry time on requests. Can be a human
    /// readable time (like `100s`) or a number of seconds.
    #[clap(long)]
    ttl: Option<humantime::Duration>,
    #[clap(subcommand)]
    sub: Cmd,
}

#[derive(Clap)]
enum Cmd {
    /// Send a query call
    Query(Args),
    /// Send a update call
    Update(Args),
}

#[derive(Clap)]
pub struct Args {
    /// Canister id
    canister_id: String,
    /// Method name to call on the canister
    method_name: String,
    /// Argument to pass to the method, in Candid textual format
    argument: Option<String>,
}

#[async_trait]
impl AsyncCommand for CallSubCommands {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        match &self.sub {
            Cmd::Query(args) => query::async_exec(args, &self, env).await,
            Cmd::Update(args) => update::async_exec(args, &self, env).await,
        }
    }
}
