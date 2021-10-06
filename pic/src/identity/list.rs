use crate::lib::identity_store::IdentityStore;
use anyhow::Result;

pub fn run() -> Result<()> {
    let store = IdentityStore::lock()?;

    for name in store.identity_names() {
        let principal = store.get_identity(name.as_str()).unwrap().sender().unwrap();
        println!("{}  {}", principal, name);
    }

    Ok(())
}
