use crate::lib::identity_store::IdentityStore;
use anyhow::{bail, Result};
use dirs::config_dir;
use ic_agent::Identity;
use std::cell::RefCell;

pub static MAIN_IC_NETWORK: &str = "https://ic0.app/";

pub struct Env {
    network: String,
    ic_server: RefCell<Option<String>>,
    identity: String,
    identity_store: IdentityStore,
}

impl Env {
    /// Create a new env.
    pub fn new(network: String, identity: Option<&str>) -> Result<Self> {
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
            network,
            ic_server: RefCell::new(None),
            identity,
            identity_store,
        })
    }

    /// Return a mutable reference to the identity store
    pub fn get_identity_store_mut(&mut self) -> &mut IdentityStore {
        &mut self.identity_store
    }

    /// Return the network that should be used.
    pub fn ic_url(&self) -> Result<String> {
        let mut net = self.ic_server.borrow_mut();

        if net.is_none() {
            let value = parse_network(self.network.as_str())?;
            net.insert(value);
        }

        Ok(net.as_ref().unwrap().to_owned())
    }

    /// Returns `true` if the current replica that should be used is the IC main network.
    pub fn is_ic_net(&self) -> Result<bool> {
        Ok(self.ic_url()? == MAIN_IC_NETWORK)
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

fn parse_network(network: &str) -> Result<String> {
    match network {
        "ic" => Ok(MAIN_IC_NETWORK.to_owned()),
        "local" => get_local_network(),
        network => Ok(network.to_owned()),
    }
}

fn get_local_network() -> Result<String> {
    todo!()
}
