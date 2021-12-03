use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

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
        todo!()
    }
}
