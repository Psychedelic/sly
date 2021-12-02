use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityUseOpts {
    /// Name of the identity to be used as default.
    name: String,
}

impl Command for IdentityUseOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        env.get_identity_store_mut().set_current(&self.name)
    }
}
