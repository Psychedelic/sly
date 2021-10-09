use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityImportOpts {
    /// Name of the new generated identity.
    name: String,
    /// Path to the PEM file.
    pem: String,
}

impl Command for IdentityImportOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut store = IdentityStore::lock()?;
        store.import(&self.name, &self.pem)
    }
}
