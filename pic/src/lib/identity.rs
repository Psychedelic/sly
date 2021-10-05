use anyhow::{bail, Context};
use ic_agent::identity::BasicIdentity;
use ic_agent::Identity;
use mkdirp::mkdirp;
use pem::{encode, Pem};
use ring::signature::Ed25519KeyPair;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

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
    /// Read the directory and load the identity store.
    fn load(directory: PathBuf) -> anyhow::Result<Self> {
        // Make sure the directory exists.
        mkdirp(directory.clone())?;

        // Read the identities from the directory.
        let mut identities = BTreeMap::<String, Box<dyn Identity>>::new();
        for (key, file_path) in glob_pem_files(&directory)? {
            let identity = BasicIdentity::from_pem_file(&file_path)
                .with_context(|| format!("Could not load pem file: {:?}", file_path))?;
            identities.insert(key, Box::new(identity));
        }

        // Read the config to maybe_config.
        let mut maybe_config = None;
        let config_json_path = directory.join("config.json");
        if let Ok(content) = fs::read_to_string(&config_json_path) {
            if let Ok(config) = serde_json::from_str::<Config>(&content) {
                maybe_config = Some(config);
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
            store
                .create("default")
                .context("Creation of default identity failed.")?;
        }

        // Determines if we need to switch to the new identity.
        let switch_to_default = match &maybe_config {
            Some(config) if store.identities.contains_key(&config.default) => false,
            _ => true,
        };

        if switch_to_default {
            let default = store.identities.keys().next().unwrap().clone();
            let config = Config { default };
            // Write the new config.
            fs::write(&config_json_path, serde_json::to_string(&config).unwrap())
                .context("Failed to write the identity config.")?;
            maybe_config = Some(config);
        }

        // Set the name of the current identity.
        store.current = maybe_config.unwrap().default;

        Ok(store)
    }

    /// Create a new randomly generated identity and store it with the given name.
    pub fn create(&mut self, name: &str) -> anyhow::Result<()> {
        if self.identities.contains_key(name) {
            bail!("Duplicate identity name {}.", name);
        }

        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Could not generate a key pair.")
            .as_ref()
            .to_vec();

        let pem = Pem {
            tag: name.to_string(),
            contents: pkcs8_bytes,
        };

        let pem_contents = encode(&pem);
        let pem_filename = format!("{}.pem", name);
        let pem_path = self.directory.join(pem_filename);

        let key_pair = Ed25519KeyPair::from_pkcs8(&pem.contents)?;
        let identity = BasicIdentity::from_key_pair(key_pair);

        fs::write(pem_path, pem_contents)?;

        self.identities.insert(name.into(), Box::new(identity));

        Ok(())
    }

    pub fn import(&mut self, name: &str, pem_path: &str) {}

    pub fn remove(&mut self, name: &str) {}

    pub fn rename(&mut self, name: &str, new_name: &str) {}

    pub fn set_current(&mut self, name: &str) {}

    /// Return the name of the current identity that is being used.
    pub fn get_current(&self) -> &str {
        &self.current
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
