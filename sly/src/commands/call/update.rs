use crate::commands::call::helper;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;
use anyhow::Result;
use async_trait::async_trait;
use clap::Clap;

#[derive(Clap, Debug)]
pub struct UpdateOpts {
    /// Canister id
    canister_id: String,
    /// Method name to call on the canister
    method_name: String,
    /// Argument to pass to the method, in Candid textual format
    argument: Option<String>,
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
}

#[async_trait]
impl AsyncCommand for UpdateOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        Ok(())
    }
}
