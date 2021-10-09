use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityWhoamiOpts {}

impl Command for IdentityWhoamiOpts {
    fn exec(self, env: &Env) -> Result<()> {
        println!("{}", env.identity_name());
        Ok(())
    }
}
