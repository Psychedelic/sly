use std::path::PathBuf;

use anyhow::{bail, Context, Result};

/// Return the path to the directory which current version of user's dfx uses to store
/// the different toolchain binaries.
pub fn get_dfx_bin_root() -> Result<PathBuf> {
    if is_version_installed("0.8.1")? {
        get_bin_cache("0.8.1")
    } else if is_version_installed("0.8.0")? {
        get_bin_cache("0.8.0")
    } else {
        bail!("Can not find DFX toolchains.")
    }
}

fn is_version_installed(v: &str) -> Result<bool> {
    get_bin_cache(v).map(|c| c.is_dir())
}

fn get_bin_cache(v: &str) -> Result<PathBuf> {
    Ok(get_bin_cache_root()?.join(v))
}

fn get_bin_cache_root() -> Result<PathBuf> {
    Ok(get_cache_root()?.join("versions"))
}

fn get_cache_root() -> Result<PathBuf> {
    let config_root = std::env::var("DFX_CONFIG_ROOT").ok();
    let home = std::env::var("HOME").context("Can not find the home directory.")?;
    let root = config_root.unwrap_or(home);
    let p = PathBuf::from(root).join(".cache").join("dfinity");
    if !p.is_dir() {
        bail!("DFX is not installed.");
    }
    Ok(p)
}
