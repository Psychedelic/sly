use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct IdentityListOpts {}

impl Command for IdentityListOpts {
    fn exec(self, env: &mut Env) -> Result<()> {
        let store = env.get_identity_store_mut();

        for name in store.identity_names() {
            let identity = store.get_identity(name.as_str()).unwrap();
            let principal = identity.sender().unwrap();
            println!("{}  {}", principal, name);
        }

        Ok(())
    }
}
