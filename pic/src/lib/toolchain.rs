use crate::lib::dfx::get_dfx_bin_root;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Return the path for the given toolchain binary.
pub fn get_binary_command_path(binary_name: &str) -> Result<PathBuf> {
    // TODO(qti3e) Don't always rely on DFX to be present on system.
    let root_bin_path = get_dfx_bin_root().context(unindent::unindent(
        "We're currently BETA. So we didn't had time to include the toolchain \
            in our binaries. Please install either DFX 0.8.0 or 0.8.1 and run a replica \
            with it only once to force it to produce the toolchains for you.",
    ))?;
    Ok(root_bin_path.join(binary_name))
}
