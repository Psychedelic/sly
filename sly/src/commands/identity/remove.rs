use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityRemoveOpts {
    /// Name of the identity to be removed.
    name: String,
}

impl Command for IdentityRemoveOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        env.get_identity_store_mut().remove(&self.name)
    }
}
