use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityCreateOpts {
    /// Name of the new generated identity.
    name: String,
}

impl Command for IdentityCreateOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut store = IdentityStore::lock()?;
        store.create(&self.name)
    }
}
