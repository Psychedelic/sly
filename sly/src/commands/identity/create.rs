use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

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
