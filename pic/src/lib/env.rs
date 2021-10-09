use crate::lib::identity_store::IdentityStore;
use anyhow::{anyhow, Result};

pub static MAIN_IC_NETWORK: &str = "https://ic0.app/";

pub struct Env {
    network: String,
    identity: String,
}

impl Env {
    /// Create a new env.
    pub fn new(network: Option<&str>, identity: Option<&str>) -> Result<Self> {
        Ok(Self {
            network: parse_network(network)?,
            identity: parse_identity(identity)?,
        })
    }

    /// Return the network that should be used.
    pub fn network(&self) -> &str {
        self.network.as_str()
    }

    /// Return the name of the identity that should be used.
    pub fn identity_name(&self) -> &str {
        self.identity.as_str()
    }
}

fn parse_network(network: Option<&str>) -> Result<String> {
    match network {
        Some("ic") => Ok(MAIN_IC_NETWORK.to_owned()),
        None | Some("local") => get_local_network(),
        Some(network) => Ok(network.to_owned()),
    }
}

fn parse_identity(identity: Option<&str>) -> Result<String> {
    let store = IdentityStore::lock()?;

    match identity {
        Some(name) if store.get_identity(name).is_some() => Ok(name.to_owned()),
        Some(name) => Err(anyhow!("Identity '{}' does not exists.", name)),
        None => Ok(store.get_current_identity_name().to_owned()),
    }
}

fn get_local_network() -> Result<String> {
    todo!()
}
