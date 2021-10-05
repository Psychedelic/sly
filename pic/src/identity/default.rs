use crate::lib::identity::IdentityStore;
use anyhow::Result;

pub fn run(name: &str) -> Result<()> {
    let mut store = IdentityStore::lock()?;
    store.set_current(name)
}
