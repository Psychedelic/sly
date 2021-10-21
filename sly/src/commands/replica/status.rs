use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::Clap;
use ic_agent::agent;
use ic_agent::agent::ReplicaV2Transport;

#[derive(Clap)]
pub struct ReplicaStatusOpts {}

#[async_trait]
impl AsyncCommand for ReplicaStatusOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        let url = env.ic_url()?;
        let transport = agent::http_transport::ReqwestHttpReplicaV2Transport::create(&url)
            .context("Failed to create Transport for Agent")?;

        let bytes = transport.status().await?;
        println!("Status bytes(len={}) {:?}", bytes.len(), bytes);

        Ok(())
    }
}
