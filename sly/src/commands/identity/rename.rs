use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityRenameOpts {
    /// Original name of the identity you want to change.
    name: String,
    /// The name for the given identity.
    new_name: String,
}

impl Command for IdentityRenameOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        env.get_identity_store_mut()
            .rename(&self.name, &self.new_name)
    }
}
