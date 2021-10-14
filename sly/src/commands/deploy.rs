use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct DeployOpts {
    /// The canister to deploy.
    canister: Option<String>,
    /// The installation mode.
    #[clap(short, long, possible_values=&(["install","reinstall","upgrade"]), default_value = "install")]
    mode: String,
}

impl Command for DeployOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        todo!()
    }
}
