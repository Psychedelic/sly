use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::lib::dfx::get_dfx_bin_root;

/// Return the path for the given toolchain binary.
pub fn get_binary_command_path(binary_name: &str) -> Result<PathBuf> {
    // TODO(qti3e) Don't always rely on DFX to be present on system.
    let root_bin_path = get_dfx_bin_root().context(unindent::unindent(
        "We're currently BETA. So we didn't had time to include the toolchain \
            in our binaries. Please install either DFX 0.8.0 or 0.8.1 and run a replica \
            with it only once to force it to produce the toolchains for you.",
    ))?;
    let path = root_bin_path.join(binary_name);
    if !path.is_file() {
        bail!(
            "Cannot obtain the path to the toolchain's {} command",
            binary_name
        );
    }
    Ok(path)
}

/// Return the file that ic-starter should write its port to.
pub fn get_replica_port_file() -> Result<PathBuf> {
    Ok(get_replica_state_root()?.join("replica-port"))
}

/// Return the file that replica actor uses to store the ic-starter's pid.
pub fn get_replica_pid_file() -> Result<PathBuf> {
    Ok(get_replica_state_root()?.join("replica-pid"))
}

/// The directory that is used by the ic-starter to store the replicated_state.
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

/// Return the directory that is used by the replica to store the state and pid/port files.
/// This method ensures that the directory does exists and creates the directory in case it
/// does not exists.
fn get_replica_state_root() -> Result<PathBuf> {
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
