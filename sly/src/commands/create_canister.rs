use async_trait::async_trait;
use clap::Parser as Clap;

use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct CreateCanisterOpts {
    /// Create a canister for all the canisters in sly.json.
    #[clap(long)]
    all: bool,
    /// The canister to create.
    canisters: Vec<String>,
}

#[async_trait]
impl AsyncCommand for CreateCanisterOpts {
    async fn async_exec(self, env: &mut Env) -> anyhow::Result<()> {
        todo!()
    }
}
