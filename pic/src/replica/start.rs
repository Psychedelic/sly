use crate::actors::{start_replica, start_shutdown_controller};
use actix::{Actor, System};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let system = System::new();

    system.block_on(async {
        let shutdown_controller = start_shutdown_controller()?;
        start_replica(Some(shutdown_controller), true)
    })?;

    system.run()?;

    Ok(())
}
