use crate::lib::identity_store::IdentityStore;
use anyhow::Result;

pub fn run(name: &str, pem_path: &str) -> Result<()> {
    let mut store = IdentityStore::lock()?;
    store.import(name, pem_path)
}
