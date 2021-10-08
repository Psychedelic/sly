use actix::{Actor, Addr};
use anyhow::Result;

pub mod child;
pub mod icx_proxy;
pub mod replica;
pub mod shutdown;
pub mod shutdown_controller;

use shutdown_controller::ShutdownController;

/// Start a shutdown controller actor.
pub fn start_shutdown_controller() -> Result<Addr<ShutdownController>> {
    Ok(ShutdownController::new().start())
}
