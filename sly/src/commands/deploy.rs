use crate::Command;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser as Clap;

use crate::commands::build::BuildOpts;
use crate::commands::create_canister::CreateCanisterOpts;
use crate::commands::install_code::InstallOpts;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct DeployOpts {
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
impl AsyncCommand for DeployOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        let create_opts = CreateCanisterOpts {
            all: self.all,
            canisters: self.canisters.clone(),
        };

        let build_opts = BuildOpts {
            with_mode: self.with_mode.clone(),
            all: self.all,
            canisters: self.canisters.clone(),
        };

        let install_opts = InstallOpts {
            mode: self.mode.clone(),
            with_mode: self.with_mode.clone(),
            all: self.all,
            canisters: self.canisters.clone(),
        };

        create_opts
            .async_exec(env)
            .await
            .context("Encountered error in create canisters step")?;
        build_opts
            .exec(env)
            .context("Encountered error in build canisters step")?;
        install_opts
            .async_exec(env)
            .await
            .context("Encountered error in install canisters step")?;

        Ok(())
    }
}
