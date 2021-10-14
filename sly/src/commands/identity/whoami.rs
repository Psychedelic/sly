use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityWhoamiOpts {}

impl Command for IdentityWhoamiOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        println!("{}", env.current_identity_name());
        Ok(())
    }
}
