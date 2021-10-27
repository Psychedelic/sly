use actix::System;
use anyhow::Result;
use async_trait::async_trait;

use crate::lib::env::Env;

pub trait Command {
    /// Execute the command.
    fn exec(self, env: &mut Env) -> Result<()>;
}

#[async_trait]
pub trait AsyncCommand {
    const RUN_SYSTEM: bool = false;

    /// Execute the command.
    async fn async_exec(self, env: &mut Env) -> Result<()>;

    /// Execute the async command and wait for the result.
    fn exec(self, env: &mut Env) -> Result<()>
    where
        Self: Sized,
    {
        let system = System::new();

        system.block_on(self.async_exec(env))?;

        if Self::RUN_SYSTEM {
            system.run()?;
        }

        Ok(())
    }
}
