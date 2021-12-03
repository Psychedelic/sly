use async_trait::async_trait;
use clap::Parser as Clap;

use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct InstallOpts {
    /// The installation mode.
    #[clap(short, long, possible_values = & (["install", "reinstall", "upgrade"]), default_value = "install")]
    mode: String,
    /// For conditional sly.json evaluation.
    #[clap(long, default_value = "default")]
    with_mode: String,
    /// Install the code for all of the canisters in sly.json.
    #[clap(long)]
    all: bool,
    /// The canister to install.
    canisters: Vec<String>,
}

#[async_trait]
impl AsyncCommand for InstallOpts {
    async fn async_exec(self, env: &mut Env) -> anyhow::Result<()> {
        todo!()
    }
}