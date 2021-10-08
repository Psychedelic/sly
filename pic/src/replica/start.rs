use crate::actors::replica::{ReplicaActor, ReplicaActorConfig};
use crate::actors::start_shutdown_controller;
use crate::lib::toolchain::get_binary_command_path;
use actix::{Actor, System};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let replica_path = get_binary_command_path("replica")?;
    let ic_starter_path = get_binary_command_path("ic-starter")?;

    let system = System::new();

    system.block_on(async {
        let shutdown_controller =
            start_shutdown_controller().expect("Cannot start the shutdown actor.");
        ReplicaActor::new(ReplicaActorConfig {
            shutdown_controller,
        })
        .start()
    });

    system.run()?;

    Ok(())
}
