use crate::actors::{start_replica, start_shutdown_controller};
use crate::lib::command::Command;
use crate::lib::env::Env;
use actix::System;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct ReplicaStartOpts {
    /// Removes the artificial delay in the local replica added to simulate the networked IC environment.
    #[clap(long)]
    no_artificial_delay: bool,
}

impl Command for ReplicaStartOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        let system = System::new();

        system.block_on(async {
            let shutdown_controller = start_shutdown_controller()?;
            start_replica(Some(shutdown_controller), self.no_artificial_delay)
        })?;

        system.run()?;

        Ok(())
    }
}
