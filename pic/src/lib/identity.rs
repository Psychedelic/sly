use anyhow::{bail, Context};
use dirs::config_dir;
use ic_agent::Identity;
use mkdirp::mkdirp;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use ic_agent::identity::Secp256k1Identity;
use openssl::{
    ec::{EcGroup, EcKey},
    nid::Nid,
    pkey::Private
};

static STORE: Lazy<Mutex<IdentityStore>> = Lazy::new(|| {
    let dir = config_dir()
        .expect("Cannot find the config dir.")
        .join("pic")
        .join("identities");
    let identity = IdentityStore::load(dir).expect("Failed to init the identity store.");
    Mutex::new(identity)
});

/// A data store that keeps the identities loaded by a user.
pub struct IdentityStore {
    directory: PathBuf,
    current: String,
    identities: BTreeMap<String, Box<dyn Identity>>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Name of the default identity to be used.
    default: String,
}

impl IdentityStore {
    /// Acquire a mutex lock on the IdentityStore.
    pub fn lock() -> anyhow::Result<MutexGuard<'static, IdentityStore>> {
        match STORE.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => bail!("Can not acquire the lock on the identity store {}", e),
        }
    }

    /// Load an identity store from the given path or init one if it doesn't already exists.
    pub fn load(directory: PathBuf) -> anyhow::Result<Self> {
        log::trace!(
            "Loading the identity store from the directory {:?}",
            directory
        );
        // Make sure the directory exists.
        mkdirp(directory.clone())?;

        // Read the identities from the directory.
        let mut identities = BTreeMap::<String, Box<dyn Identity>>::new();
        for (key, file_path) in glob_pem_files(&directory)? {
            log::trace!(
                "Loading pem file for identity '{}' from {:?}",
                key,
                file_path
            );
            let identity = Secp256k1Identity::from_pem_file(&file_path)
                .with_context(|| format!("Could not load pem file: {:?}", file_path))?;
            identities.insert(key, Box::new(identity));
        }

        // Read the config to maybe_config.
        let mut maybe_config = None;
        let config_json_path = directory.join("config.json");
        if let Ok(content) = fs::read_to_string(&config_json_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&content) {
                maybe_config = Some(config);
            } else {
                log::error!("Invalid config file was found.")
            }
        }

        // Create a tmp store we use to create the default identity on in case it does not
        // already exists.
        let mut store = Self {
            directory,
            current: "".into(),
            identities,
        };

        // Create the default identity if it doesn't already exists.
        if store.identities.is_empty() {
            log::trace!("Creating default identity because no identity exists.");
            store
                .create("default")
                .context("Creation of default identity failed.")?;
        }

        match maybe_config {
            Some(config) if store.identities.contains_key(&config.default) => {
                store.current = config.default;
            }
            _ => {
                let default = store.identities.keys().next().unwrap().clone();
                log::info!(
                    "Could not load the default identity. Changing the default identity to '{}'",
                    default
                );
                store.set_current(default.as_str())?;
            }
        };

        Ok(store)
    }

    /// Create a new randomly generated identity and store it with the given name.
    pub fn create(&mut self, name: &str) -> anyhow::Result<()> {
        log::trace!("Creating new identity '{}'", name);
        if self.identities.contains_key(name) {
            bail!("Duplicate identity name {}.", name);
        }

        let group = EcGroup::from_curve_name(Nid::SECP256K1)?;
        let private_key = EcKey::generate(&group).unwrap();

        self.save_secp256k1_pem(name, private_key)
    }

    /// Import a new identity from a pem file.
    pub fn import(&mut self, name: &str, pem_path: &str) -> anyhow::Result<()> {
        log::trace!("Importing new identity '{}' from pem '{}'", name, pem_path);
        if self.identities.contains_key(name) {
            bail!("Duplicate identity name {}.", name);
        }

        log::trace!("Reading the pem file {}", pem_path);
        let pem = fs::read(pem_path).context("Cannot read the pem file.")?;
        log::trace!("Parsing the pem file");
        let private_key = EcKey::private_key_from_pem(&pem)?;

        self.save_secp256k1_pem(name, private_key)
    }

    fn save_secp256k1_pem(
        &mut self,
        name: &str,
        private_key: EcKey<Private>,
    ) -> anyhow::Result<()> {
        let pem_contents = private_key.private_key_to_pem()?;
        let pem_filename = format!("{}.pem", name);
        let pem_path = self.directory.join(pem_filename);

        let identity = Secp256k1Identity::from_private_key(private_key);

        fs::write(pem_path, pem_contents)?;
        self.identities.insert(name.into(), Box::new(identity));

        Ok(())
    }

    /// Remove the identity with the given name.
    pub fn remove(&mut self, name: &str) -> anyhow::Result<()> {
        log::trace!("Removing identity '{}'", name);
        if !self.identities.contains_key(name) {
            bail!("Can not find identity {}", name);
        }

        let path = self.directory.join(format!("{}.pem", name));
        fs::remove_file(path)?;

        self.identities.remove(name);

        if self.current == name {
            if self.identities.is_empty() {
                log::trace!("All of the identities are removed. Creating a new default.");
                self.create("default")?;
            }

            let default = self.identities.keys().next().unwrap().clone();
            log::trace!(
                "Default identity was removed. Changing the default identity to {}",
                default
            );
            self.set_current(default.as_str())?;
        }

        Ok(())
    }

    /// Rename an identity to its new name.
    pub fn rename(&mut self, name: &str, new_name: &str) -> anyhow::Result<()> {
        log::trace!("Renaming identity '{}' to '{}'", name, new_name);
        if !self.identities.contains_key(name) {
            bail!("Can not find identity {}", name);
        }

        if self.identities.contains_key(new_name) {
            bail!("Duplicate identity name {}.", new_name);
        }

        // Perform the local rename.
        let identity = self.identities.remove(name).unwrap();
        self.identities.insert(new_name.into(), identity);

        // Rename the file.
        let from_path = self.directory.join(format!("{}.pem", name));
        let to_path = self.directory.join(format!("{}.pem", new_name));
        fs::rename(from_path, to_path)?;

        // If we renamed the default identity, also update the config.
        if self.current == name {
            self.set_current(new_name)?;
        }

        Ok(())
    }

    /// Set the default identity that should be used.
    pub fn set_current(&mut self, name: &str) -> anyhow::Result<()> {
        log::trace!("Setting the current identity to {}", name);

        if !self.identities.contains_key(name) {
            bail!("Can not find identity {}", name);
        }

        let config_json_path = self.directory.join("config.json");
        let config = Config {
            default: name.into(),
        };
        fs::write(&config_json_path, serde_json::to_string(&config).unwrap())
            .context("Failed to write the identity config.")?;

        self.current = name.into();
        Ok(())
    }

    /// Return the name of the current identity that is being used.
    pub fn get_current_identity_name(&self) -> &str {
        &self.current
    }

    /// Return the default identity that should be used.
    pub fn get_current_identity(&self) -> &Box<dyn Identity> {
        self.identities.get(&self.current).unwrap()
    }

    /// Return an iterator over the name of all the loaded identities.
    pub fn identity_names(&self) -> impl Iterator<Item = &String> {
        self.identities.keys()
    }

    /// Return the identity by name.
    pub fn get_identity(&self, name: &str) -> Option<&Box<dyn Identity>> {
        self.identities.get(name)
    }
}

/// Create an iterator over (IdentityName, PemFilePath) of all the pem files in a directory.
fn glob_pem_files(directory: &PathBuf) -> anyhow::Result<impl Iterator<Item = (String, PathBuf)>> {
    Ok(fs::read_dir(directory)?.filter_map(|entry| {
        if let Ok(entry) = entry {
            let file_path = entry.path();

            if !file_path.is_file() {
                return None;
            }

            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".pem") {
                    let name = name[0..name.len() - 4].to_string();
                    return Some((name, file_path));
                }
            }
        }

        None
    }))
}
