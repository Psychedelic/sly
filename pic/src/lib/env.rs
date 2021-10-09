use crate::lib::identity_store::IdentityStore;
use anyhow::{bail, Result};
use dirs::config_dir;
use ic_agent::Identity;

pub static MAIN_IC_NETWORK: &str = "https://ic0.app/";

pub struct Env {
    network: String,
    identity: String,
    identity_store: IdentityStore,
}

impl Env {
    /// Create a new env.
    pub fn new(network: Option<&str>, identity: Option<&str>) -> Result<Self> {
        let directory = config_dir()
            .expect("Cannot find the config dir.")
            .join("psychedelic")
            .join("identities");

        let identity_store = IdentityStore::load(directory)?;

        let identity = if let Some(name) = identity {
            if identity_store.get_identity(name).is_none() {
                bail!("Identity '{}' does not exists.", name);
            }

            name.to_owned()
        } else {
            identity_store.get_current_identity_name().to_owned()
        };

        Ok(Self {
            network: parse_network(network)?,
            identity,
            identity_store,
        })
    }

    /// Return a mutable reference to the identity store
    pub fn get_identity_store_mut(&mut self) -> &mut IdentityStore {
        &mut self.identity_store
    }

    /// Return the network that should be used.
    pub fn network(&self) -> &str {
        self.network.as_str()
    }

    /// Return the name of the identity that should be used.
    pub fn current_identity_name(&self) -> &str {
        self.identity.as_str()
    }

    /// Return the current identity that should be used.
    pub fn current_identity(&self) -> &Box<dyn Identity> {
        let name = self.identity.as_str();
        self.identity_store.get_identity(name).unwrap()
    }
}

fn parse_network(network: Option<&str>) -> Result<String> {
    match network {
        Some("ic") => Ok(MAIN_IC_NETWORK.to_owned()),
        None | Some("local") => get_local_network(),
        Some(network) => Ok(network.to_owned()),
    }
}

fn get_local_network() -> Result<String> {
    todo!()
}
