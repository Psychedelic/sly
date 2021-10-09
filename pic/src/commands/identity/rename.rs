use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityRenameOpts {
    /// Original name of the identity you want to change.
    name: String,
    /// The name for the given identity.
    new_name: String,
}

impl Command for IdentityRenameOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut store = IdentityStore::lock()?;
        store.rename(&self.name, &self.new_name)
    }
}
