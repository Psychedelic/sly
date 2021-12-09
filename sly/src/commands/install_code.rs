use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use candid::Principal;
use clap::Parser as Clap;
use futures::future::join_all;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::commands::call::waiter;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct InstallOpts {
    /// The installation mode.
    #[clap(short, long, possible_values = & (["install", "reinstall", "upgrade"]), default_value = "install")]
    pub mode: String,
    /// For conditional sly.json evaluation.
    #[clap(long, default_value = "default")]
    pub with_mode: String,
    /// Install the code for all of the canisters in sly.json.
    #[clap(long)]
    pub all: bool,
    /// The canister to install.
    pub canisters: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CanisterIdJson(BTreeMap<String, BTreeMap<String, Principal>>);

#[async_trait]
impl AsyncCommand for InstallOpts {
    async fn async_exec(self, env: &mut Env) -> anyhow::Result<()> {
        if !self.canisters.is_empty() && self.all {
            bail!("Unexpect canisters list when --all is provided.");
        }

        let workspace = env.workspace()?;
        let host = env.network();
        let filename = if host == "local" {
            "canister_ids-local.json"
        } else {
            "canister_ids.json"
        };

        let json_path = workspace.root.join(filename);
        let json = std::fs::read_to_string(json_path).with_context(|| {
            format!(
                "Could not read {}. Run 'sly create_canister' first.",
                filename
            )
        })?;
        let canister_ids = serde_json::from_str::<CanisterIdJson>(&json)
            .with_context(|| format!("Could not parse {}", filename))?;

        let canisters = if self.all {
            workspace.canisters.keys().cloned().collect()
        } else {
            self.canisters.clone()
        };

        let mut to_install = vec![];

        for name in canisters.clone() {
            let canister = workspace
                .get_canister(&name)
                .ok_or_else(|| anyhow!("Canister '{}' not found.", name))?;

            let canister_id = canister_ids
                .0
                .get(&name)
                .map(|m| m.get(&host))
                .flatten()
                .ok_or_else(|| {
                    anyhow!(
                        "Canister '{}' is not created. Please use sly create first.",
                        name
                    )
                })?;

            let wasm_path = canister.wasm.get(&self.with_mode).ok_or_else(|| {
                anyhow!(
                    "Canister '{}' does not have a wasm field for mode '{}'.",
                    name,
                    self.with_mode
                )
            })?;

            let wasm_path = workspace.root.join(PathBuf::from(wasm_path));

            if !wasm_path.is_file() {
                bail!(
                    "Could not locate WASM for canister '{}'. '{}' is not a file.",
                    name,
                    wasm_path.to_string_lossy()
                )
            }

            let wasm = std::fs::read(&wasm_path)
                .with_context(|| format!("Could not read '{}'", wasm_path.to_string_lossy()))?;

            to_install.push((*canister_id, wasm));
        }

        let agent = env.create_agent().await?;

        let futures = to_install
            .into_iter()
            .map(|(canister_id, wasm)| install_code(&agent, canister_id, wasm))
            .collect::<Vec<_>>();

        let mut had_error = false;
        for (name, result) in canisters.into_iter().zip(join_all(futures).await) {
            if let Err(e) = result {
                had_error = true;
                log::error!("Failed to install code for canister '{}': {}", name, e);
            }
        }

        if had_error {
            bail!("Failed to install code for some of the canisters.");
        }

        Ok(())
    }
}

async fn install_code(agent: &Agent, canister_id: Principal, wasm: Vec<u8>) -> anyhow::Result<()> {
    ManagementCanister::create(agent)
        .install_code(&canister_id, &wasm)
        .build()
        .unwrap()
        .call_and_wait(waiter::waiter_with_exponential_backoff())
        .await?;

    Ok(())
}
