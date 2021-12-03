use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityImportOpts {
    /// Name of the new generated identity.
    name: String,
    /// Path to the PEM file.
    pem: String,
}

impl Command for IdentityImportOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        env.get_identity_store_mut().import(&self.name, &self.pem)
    }
}
