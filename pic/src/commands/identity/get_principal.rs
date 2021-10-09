use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityGetPrincipalOpts {}

impl Command for IdentityGetPrincipalOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        let principal = env.current_identity().sender().unwrap();
        println!("{}", principal);
        Ok(())
    }
}
