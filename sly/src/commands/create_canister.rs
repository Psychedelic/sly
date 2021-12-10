use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use clap::Parser as Clap;
use futures::future::join_all;
use ic_agent::ic_types::Principal;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::{ManagementCanister, Wallet};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{Read, Write};

use crate::commands::call::waiter;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct CreateCanisterOpts {
    /// Create a canister for all the canisters in sly.json.
    #[clap(long)]
    pub all: bool,
    /// The canister to create.
    pub canisters: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CanisterIdJson(BTreeMap<String, BTreeMap<String, Principal>>);

#[async_trait]
impl AsyncCommand for CreateCanisterOpts {
    async fn async_exec(self, env: &mut Env) -> anyhow::Result<()> {
        if !self.canisters.is_empty() && self.all {
            bail!("Unexpect canisters list when --all is provided.");
        }

        let workspace = env.workspace()?;
        let host = env.network();
        let use_provisional = host == "local";

        // Use a different file for local env so people can gitignore it.
        let filename = if host == "local" {
            "canister_ids-local.json"
        } else {
            "canister_ids.json"
        };

        for name in &self.canisters {
            workspace
                .get_canister(name)
                .ok_or_else(|| anyhow!("Canister '{}' not found.", name))?;
        }

        let json_path = workspace.root.join(filename);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(json_path)
            .with_context(|| format!("Failed to open {}", filename))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .with_context(|| format!("Failed to read {}", filename))?;

        let mut canister_ids = if json.is_empty() {
            CanisterIdJson(BTreeMap::new())
        } else {
            serde_json::from_str::<CanisterIdJson>(&json)
                .with_context(|| format!("Failed to parse {}", filename))?
        };

        // Name of the canisters we should create a canister for.
        let canisters = if self.all {
            workspace.canisters.keys().cloned().collect()
        } else {
            self.canisters.clone()
        };

        let agent = env.create_agent().await?;

        let to_create = canisters
            .into_iter()
            .filter_map(|name| canister_ids.0.get(&name)?.get(&host).and(Some(name)))
            .collect::<Vec<_>>();

        let futures = to_create
            .iter()
            .map(|_| create_canister(use_provisional, &agent));
        let new_canister_ids = join_all(futures).await;
        let mut had_error = false;

        for (name, maybe_principal) in to_create.into_iter().zip(new_canister_ids) {
            let canister_ids = canister_ids.0.entry(name).or_default();
            match maybe_principal {
                Ok(principal) => {
                    canister_ids.insert(host.clone(), principal);
                }
                Err(e) => {
                    had_error = true;
                    log::error!("Could not create canister: {}", e);
                }
            }
        }

        let json = serde_json::to_string_pretty(&canister_ids)
            .context("Failed to serialize canister ids.")?;
        file.write(json.as_bytes())
            .with_context(|| format!("Failed to write {}'s content.", filename))?;
        file.set_len(json.len() as u64)
            .with_context(|| format!("Failed to truncate {}", filename))?;

        if had_error {
            bail!("Some of the canisters were not created.")
        }

        Ok(())
    }
}

async fn create_canister(use_provisional: bool, agent: &Agent) -> anyhow::Result<Principal> {
    if use_provisional {
        log::trace!("Creating a canister using provisional_create_canister_with_cycles");

        let management = ManagementCanister::create(agent);
        let (canister_id,) = management
            .create_canister()
            .as_provisional_create_with_amount(Some(100_000_000_000_000))
            .build()
            .unwrap()
            .call_and_wait(waiter::waiter_with_exponential_backoff())
            .await
            .context("provisional_create_canister_with_cycles call failed.")?;

        Ok(canister_id)
    } else {
        log::trace!("Creating a canister using XTC wallet");

        let xtc = Principal::from_text("aanaa-xaaaa-aaaah-aaeiq-cai").unwrap();
        let canister_id = Wallet::create(agent, xtc)
            .wallet_create_canister(
                4_000_000_000_000,
                None,
                None,
                None,
                None,
                waiter::waiter_with_exponential_backoff(),
            )
            .await
            .context("XTC create canister call failed.")?
            .canister_id;

        Ok(canister_id)
    }
}
