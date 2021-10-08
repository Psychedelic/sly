use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

pub fn get_replica_port_file() -> Result<PathBuf> {
    Ok(get_replica_state_root()?.join("replica-port"))
}

pub fn get_replica_pid_file() -> Result<PathBuf> {
    Ok(get_replica_state_root()?.join("replica-pid"))
}

pub fn get_replica_state_directory() -> Result<PathBuf> {
    let root = get_replica_state_root()?;
    let state_directory = root.join("state");

    if !state_directory.exists() {
        fs::create_dir_all(&state_directory)
            .context("Can not create the replica state directory")?;
    }

    if !state_directory.is_dir() {
        bail!("Expected '{:?}' to be a directory.", state_directory)
    }

    Ok(state_directory)
}

pub fn get_replica_state_root() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().context("Can not get the data directory.")?;
    let root = data_dir.join("psychedelic").join("replica");

    if !root.exists() {
        fs::create_dir_all(&root).context("Can not create the replica data directory")?;
    }

    if !root.is_dir() {
        bail!("Expected '{:?}' to be a directory.", root);
    }

    Ok(root)
}
