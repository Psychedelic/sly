use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityGetPrincipalOpts {}

impl Command for IdentityGetPrincipalOpts {
    fn exec(self, env: &Env) -> Result<()> {
        let store = IdentityStore::lock()?;
        let name = env.identity_name();
        let principal = store.get_identity(name).unwrap().sender().unwrap();
        println!("{}", principal);
        Ok(())
    }
}
