use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

/// Holds the information gathered from parsing Sly.json
#[derive(Clone)]
pub struct Workspace {
    /// The root directory of the project, this is where the
    /// sly.json file can be found.
    pub root: PathBuf,
    /// The list of canisters.
    pub canisters: BTreeMap<String, Canister>,
}

#[derive(Debug, Clone)]
pub struct Canister {
    pub build: BTreeMap<String, Vec<String>>,
    pub test: BTreeMap<String, Vec<String>>,
    pub wasm: BTreeMap<String, String>,
    pub candid: BTreeMap<String, String>,
}

impl Workspace {
    pub fn from_current_directory() -> anyhow::Result<Self> {
        let cwd = env::current_dir().context("Failed to retrieve current working directory.")?;
        let mut dir = Some(cwd.as_path());

        while let Some(root) = dir {
            let path = root.join("sly.json");

            if path.is_file() {
                let reader = std::fs::File::open(path.clone())
                    .with_context(|| format!("Failed to open file '{}'", path.to_string_lossy()))?;

                return Self::from_reader(root.into(), reader);
            }

            dir = root.parent();
        }

        bail!("No sly.json found in the current path.")
    }

    pub fn from_config_path(path: PathBuf) -> anyhow::Result<Self> {
        if !path.is_file() {
            bail!("'{}' is not a file.", path.to_string_lossy())
        }

        let root = path.parent().unwrap().into();
        let reader = std::fs::File::open(path.clone())
            .with_context(|| format!("Failed to open file '{}'", path.to_string_lossy()))?;

        Self::from_reader(root, reader)
    }

    /// Create a workspace from a Sly.json that is located in the given `root` directory.
    pub fn from_reader<R>(root: PathBuf, reader: R) -> anyhow::Result<Self>
    where
        R: std::io::Read,
    {
        let manifest = serde_json::from_reader::<_, manifest::Manifest>(reader)
            .context("Could not parse the workspace manifest file.")?;

        let canisters = manifest
            .canisters
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        Ok(Self { root, canisters })
    }

    /// Return the settings for a canister.
    pub fn get_canister<Q>(&self, name: &Q) -> Option<&Canister>
    where
        String: Borrow<Q>,
        Q: Ord,
    {
        self.canisters.get(name)
    }
}

mod manifest {
    use super::*;

    /// The schema for Sly.json files.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Manifest {
        /// The version of the replica binary which should be used
        /// for this project.
        pub version: Option<String>,
        /// List of the canisters that are developed under this
        /// project.
        pub canisters: Option<BTreeMap<String, CanisterInfo>>,
    }

    /// Information regarding a certain canister.
    #[derive(Serialize, Deserialize, Debug)]
    pub struct CanisterInfo {
        build: Option<WithMode<Command>>,
        test: Option<WithMode<Command>>,
        wasm: Option<WithMode<String>>,
        candid: Option<WithMode<String>>,
    }

    /// A type wrapper that is used for setting mode depended values
    /// for a configuration.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "build": "command"
    /// }
    /// ```
    ///
    /// Using this type we can also add support for something like this:
    ///
    /// ```json
    /// {
    ///   "build": {
    ///     "release: "release mode build command",
    ///     "debug": "debug mode build command"
    ///   }
    /// }
    /// ```
    ///
    /// And then have a command like `sly build -mode=release`
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    pub enum WithMode<T> {
        Mode(BTreeMap<String, T>),
        Data(T),
    }

    /// A command in the manifest, can be either a simple string
    /// for one command, or an array for running multiple commands.
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(untagged)]
    pub enum Command {
        Command(String),
        Commands(Vec<String>),
    }

    impl From<CanisterInfo> for Canister {
        fn from(info: CanisterInfo) -> Self {
            Self {
                build: info.build.map(|x| x.into()).unwrap_or_default(),
                test: info.test.map(|x| x.into()).unwrap_or_default(),
                wasm: info.wasm.map(|x| x.into()).unwrap_or_default(),
                candid: info.candid.map(|x| x.into()).unwrap_or_default(),
            }
        }
    }

    impl<T, U> From<WithMode<T>> for BTreeMap<String, U>
    where
        T: Into<U>,
    {
        fn from(mode: WithMode<T>) -> Self {
            match mode {
                WithMode::Mode(data) => {
                    let mut map = BTreeMap::new();
                    for (key, value) in data {
                        map.insert(key, value.into());
                    }
                    map
                }
                WithMode::Data(data) => {
                    let mut map = BTreeMap::new();
                    map.insert("default".into(), data.into());
                    map
                }
            }
        }
    }

    impl From<Command> for Vec<String> {
        fn from(cmd: Command) -> Self {
            match cmd {
                Command::Command(command) => vec![command],
                Command::Commands(commands) => commands,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_simple() {
        let manifest = serde_json::json!({
            "canisters": {
                "cap": {
                    "build": "command",
                    "wasm": "./xxx"
                }
            }
        });

        serde_json::from_value::<manifest::Manifest>(manifest).expect("Failed to deserialize.");
    }

    #[test]
    fn manifest_with_mode_command() {
        let manifest = serde_json::json!({
            "canisters": {
                "cap": {
                    "build": {
                        "release": "release command",
                        "debug": "release command"
                    }
                }
            }
        });

        serde_json::from_value::<manifest::Manifest>(manifest).expect("Failed to deserialize.");
    }

    #[test]
    fn manifest_with_mode_multiple_commands() {
        let manifest = serde_json::json!({
            "canisters": {
                "cap": {
                    "build": {
                        "release": [
                            "release command",
                            "command 2"
                        ],
                        "debug": "release command"
                    }
                }
            }
        });

        serde_json::from_value::<manifest::Manifest>(manifest).expect("Failed to deserialize.");
    }

    #[test]
    fn manifest_with_mode_path() {
        let manifest = serde_json::json!({
            "canisters": {
                "cap": {
                    "wasm": {
                        "release": "./cap-release.wasm",
                        "debug": "./cap-debug.wasm",
                    }
                }
            }
        });

        serde_json::from_value::<manifest::Manifest>(manifest).expect("Failed to deserialize.");
    }
}
