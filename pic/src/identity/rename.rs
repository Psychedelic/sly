use crate::lib::identity_store::IdentityStore;
use anyhow::Result;

pub fn run(name: &str, new_name: &str) -> Result<()> {
    let mut store = IdentityStore::lock()?;
    store.rename(name, new_name)
}
