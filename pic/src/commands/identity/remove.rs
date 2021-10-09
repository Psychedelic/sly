use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityRemoveOpts {
    /// Name of the identity to be removed.
    name: String,
}

impl Command for IdentityRemoveOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut store = IdentityStore::lock()?;
        store.remove(&self.name)
    }
}
