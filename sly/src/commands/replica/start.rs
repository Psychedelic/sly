use anyhow::Result;
use async_trait::async_trait;
use clap::Parser as Clap;

use crate::actors::{start_replica, start_shutdown_controller};
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct ReplicaStartOpts {
    /// Removes the artificial delay in the local replica added to simulate the networked IC environment.
    #[clap(long)]
    no_artificial_delay: bool,
}

#[async_trait]
impl AsyncCommand for ReplicaStartOpts {
    const RUN_SYSTEM: bool = true;

    async fn async_exec(self, _env: &mut Env) -> Result<()> {
        let shutdown_controller = start_shutdown_controller()?;
        start_replica(Some(shutdown_controller), self.no_artificial_delay)?;
        Ok(())
    }
}
