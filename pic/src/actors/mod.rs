use crate::lib::env;
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

/// Start a replica, returns the actor's address.
pub fn start_replica(
    shutdown_controller: Option<Addr<ShutdownController>>,
    no_artificial_delay: bool,
) -> Result<Addr<ReplicaActor>> {
    let replica_path = get_binary_command_path("replica")?;
    let ic_starter_path = get_binary_command_path("ic-starter")?;
    let state_directory = env::get_replica_state_directory()?;
    let write_port_to = env::get_replica_port_file()?;
    let write_pid_to = Some(env::get_replica_pid_file()?);

    let config = ReplicaActorConfig {
        ic_starter_path,
        replica_path,
        state_directory,
        write_port_to,
        write_pid_to,
        no_artificial_delay,
        shutdown_controller,
    };

    Ok(ReplicaActor::new(config).start())
}
