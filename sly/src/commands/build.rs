use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::{anyhow, bail};
use clap::Parser as Clap;
use std::process::Command as CommandExec;

#[derive(Clap)]
pub struct BuildOpts {
    /// For conditional sly.json evaluation.
    #[clap(long, default_value = "default")]
    pub with_mode: String,
    /// Build the code for all of the canisters in sly.json.
    #[clap(long)]
    pub all: bool,
    /// The canister to build.
    pub canisters: Vec<String>,
}

impl Command for BuildOpts {
    fn exec(self, env: &mut Env) -> anyhow::Result<()> {
        let workspace = env.workspace()?;
        let canisters = if self.all {
            workspace.canisters.keys().cloned().collect()
        } else {
            self.canisters.clone()
        };

        // for checking if canisters exist
        for canister in &canisters {
            if workspace.get_canister(canister).is_none() {
                bail!("Canister '{}' not found", canister);
            }
        }

        for canister in canisters {
            let commands = workspace
                .get_canister(&canister)
                .unwrap()
                .build
                .get(&self.with_mode)
                .ok_or(anyhow!("Wrong with_mode parameter"))?;

            for command in commands {
                // TODO: Shell Expand
                CommandExec::new("sh")
                    .arg("-c")
                    .arg(command)
                    .spawn()
                    .expect("SH failed to parse the command");
            }
        }
        Ok(())
    }
}
