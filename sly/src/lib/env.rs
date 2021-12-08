use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use dirs::config_dir;
use ic_agent::{agent, Agent, Identity};

use crate::lib::identity_store::IdentityStore;
use crate::lib::toolchain;
use crate::lib::workspace::Workspace;

pub static MAIN_IC_NETWORK: &str = "https://ic0.app";

pub struct Env {
    network: String,
    ic_server: Mutex<RefCell<Option<String>>>,
    workspace: Mutex<RefCell<Option<Workspace>>>,
    config_path: Option<PathBuf>,
    identity: String,
    identity_store: IdentityStore,
}

impl Env {
    /// Create a new env.
    pub fn new(
        network: String,
        identity: Option<&str>,
        config_path: Option<PathBuf>,
    ) -> Result<Self> {
        let directory = config_dir()
            .expect("Cannot find the config dir.")
            .join("psychedelic")
            .join("identities");

        let identity_store = IdentityStore::load(&directory)?;

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
            ic_server: Mutex::new(RefCell::new(None)),
            workspace: Mutex::new(RefCell::new(None)),
            config_path,
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
        let lock = self.ic_server.lock().unwrap();
        let net = lock.borrow_mut();

        if net.is_none() {
            let value = parse_network(self.network.as_str())?;
            return Ok(value
                .strip_suffix('/')
                .unwrap_or_else(|| value.as_str())
                .into());
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
    pub fn current_identity(&self) -> Box<dyn Identity> {
        let name = self.identity.as_str();
        self.identity_store.get_identity(name).unwrap()
    }

    /// Create and init the agent.
    pub async fn create_agent(&self) -> Result<Agent> {
        let url = self.ic_url()?;
        let identity = self.current_identity();

        let agent = Agent::builder()
            .with_transport(
                agent::http_transport::ReqwestHttpReplicaV2Transport::create(&url)
                    .context("Failed to create Transport for Agent")?,
            )
            .with_boxed_identity(identity)
            .build()
            .context("Failed to build the Agent")?;

        if !self.is_ic_net().unwrap() {
            log::trace!("Fetching the root key for replica: {}", url);
            agent
                .fetch_root_key()
                .await
                .context("Failed to fetch root key from replica")?;
        }

        Ok(agent)
    }

    /// Return the workspace information by parsing the sly.json file
    /// in the current directory or one of the parents.
    pub fn workspace(&self) -> anyhow::Result<Workspace> {
        let lock = self.workspace.lock().unwrap();
        let mut workspace = lock.borrow_mut();

        if workspace.is_some() {
            return Ok(workspace.as_ref().unwrap().clone());
        }

        let w = if let Some(path) = &self.config_path {
            Workspace::from_config_path(path.clone())
        } else {
            Workspace::from_current_directory()
        }
        .context("Loading sly.json failed.")?;

        *workspace = Some(w.clone());
        Ok(w)
    }

    pub fn network(&self) -> String {
        match self.network.as_str() {
            "ic" => "ic".to_string(),
            "local" => "local".to_string(),
            _ => self.ic_url().unwrap(),
        }
    }
}

fn parse_network(network: &str) -> Result<String> {
    match network {
        "ic" => Ok(MAIN_IC_NETWORK.to_owned()),
        "local" => get_local_network().context("Failed to find the address for local replica."),
        network => Ok(network.to_owned()),
    }
}

fn get_local_network() -> Result<String> {
    let port_file = toolchain::get_replica_port_file()?;

    if !port_file.is_file() {
        bail!("Local replica is not running.")
    }

    let content = std::fs::read_to_string(&port_file).context("Can not read the port file.")?;
    let port = content
        .parse::<u16>()
        .context("Can not parse the content of the port file.")?;

    Ok(format!("http://localhost:{}", port))
}
