use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::identity_store::IdentityStore;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct IdentityListOpts {}

impl Command for IdentityListOpts {
    fn exec(self, _: &Env) -> Result<()> {
        let store = IdentityStore::lock()?;

        for name in store.identity_names() {
            let principal = store.get_identity(name.as_str()).unwrap().sender().unwrap();
            println!("{}  {}", principal, name);
        }

        Ok(())
    }
}
