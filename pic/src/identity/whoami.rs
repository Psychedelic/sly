use crate::lib::identity::IdentityStore;
use anyhow::Result;

pub fn run() -> Result<()> {
    let store = IdentityStore::lock()?;
    println!("{}", store.get_current_identity_name());
    Ok(())
}
