use crate::lib::identity::IdentityStore;
use anyhow::Result;

pub fn run() -> Result<()> {
    let store = IdentityStore::lock()?;
    let principal = store.get_current_identity().sender().unwrap();
    println!("{}", principal);
    Ok(())
}
