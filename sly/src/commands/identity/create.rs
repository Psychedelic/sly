use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityCreateOpts {
    /// Name of the new generated identity.
    name: String,
}

impl Command for IdentityCreateOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        env.get_identity_store_mut().create(&self.name)
    }
}
