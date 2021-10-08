use crate::lib::toolchain::get_binary_command_path;
use actix::{Actor, Addr};
use anyhow::{bail, Context, Result};
use replica::{ReplicaActor, ReplicaActorConfig};
use shutdown_controller::ShutdownController;
use std::fs;

pub mod child_process;
pub mod icx_proxy;
pub mod replica;
pub mod shutdown;
pub mod shutdown_controller;

/// Start a shutdown controller actor.
pub fn start_shutdown_controller() -> Result<Addr<ShutdownController>> {
    Ok(ShutdownController::new().start())
}

pub fn start_replica(
    shutdown_controller: Option<Addr<ShutdownController>>,
    no_artificial_delay: bool,
) -> Result<Addr<ReplicaActor>> {
    let replica_path = get_binary_command_path("replica")?;
    let ic_starter_path = get_binary_command_path("ic-starter")?;
    let data_dir = dirs::data_dir().context("Can not get the data directory.")?;
    let state_directory_root = data_dir.join("psychedelic").join("replica");
    let state_directory = state_directory_root.join("state");

    if !state_directory.exists() {
        fs::create_dir_all(&state_directory).context("Can not create the state data directory")?;
    }

    if !state_directory.is_dir() {
        bail!("Expected '{:?}' to be a directory.", state_directory);
    }

    let config = ReplicaActorConfig {
        ic_starter_path,
        replica_path,
        state_directory,
        write_port_to: state_directory_root.join("replica-port"),
        no_artificial_delay,
        shutdown_controller,
    };

    Ok(ReplicaActor::new(config).start())
}
