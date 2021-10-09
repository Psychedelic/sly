use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityUseOpts {
    /// Name of the identity to be used as default.
    name: String,
}

impl Command for IdentityUseOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let mut store = IdentityStore::lock()?;
        store.set_current(&self.name)
    }
}
