use crate::lib::command::Command;
use crate::lib::env::Env;
use clap::Parser as Clap;
use std::process::Command as CommandExec;
use anyhow::anyhow;

#[derive(Clap)]
pub struct BuildOpts {
    /// For conditional sly.json evaluation.
    #[clap(long, default_value = "default")]
    with_mode: String,
    /// Build the code for all of the canisters in sly.json.
    #[clap(long)]
    all: bool,
    /// The canister to build.
    canisters: Vec<String>,
}

impl Command for BuildOpts {
    fn exec(self, env: &mut Env) -> anyhow::Result<()> {
        for canister in self.canisters {
            let workspace = env.workspace()?;
            let commands = workspace
                .get_canister(&canister)
                .ok_or(anyhow!("Wrong canister"))?
                .build
                .get(&self.with_mode)
                .ok_or(anyhow!("Wrong with_mode parameter"))?;

            for command in commands {
                // TODO: Shell Expand
                CommandExec::new("sh")
                .arg("-C")
                .arg(command)
                .spawn()
                .expect("SH failed to parse the command");
            }
        }
        Ok(())
    }
}
