use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Parser as Clap;

use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct ReplicaStatusOpts {}

#[async_trait]
impl AsyncCommand for ReplicaStatusOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        let agent = env.create_agent().await?;
        let status = agent
            .status()
            .await
            .context("Failed to retrieve server status.")?;

        println!("{}", status);

        Ok(())
    }
}
