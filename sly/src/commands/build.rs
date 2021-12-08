use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::{anyhow, bail, Context};
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
        if !self.canisters.is_empty() && self.all {
            bail!("Unexpect canisters list when --all is provided.");
        }

        let workspace = env.workspace()?;

        for name in &self.canisters {
            workspace
                .get_canister(name)
                .ok_or_else(|| anyhow!("Canister '{}' not found.", name))?;
        }

        let canisters = if self.all {
            workspace.canisters.keys().cloned().collect()
        } else {
            self.canisters.clone()
        };

        for name in canisters {
            let commands = workspace
                .get_canister(&name)
                .unwrap()
                .build
                .get(&self.with_mode)
                .ok_or_else(|| anyhow!("Wrong with_mode parameter"))?;

            for command in commands {
                // TODO: Shell Expand
                CommandExec::new("sh")
                    .arg("-c")
                    .arg(command)
                    .spawn()
                    .with_context(|| format!("Could not execute command '{}'", command))?;
            }
        }

        Ok(())
    }
}
