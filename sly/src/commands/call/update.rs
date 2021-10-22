use crate::commands::call::helper;
use crate::lib::command::AsyncCommand;
use crate::lib::env::Env;
use crate::lib::utils;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use candid::Principal;
use clap::Clap;
use ic_agent::{agent::agent_error::HttpErrorPayload, AgentError};
use std::time::Duration;

#[derive(Clap, Debug)]
pub struct UpdateOpts {
    /// Canister id
    canister_id: String,
    /// Method name to call on the canister
    method_name: String,
    /// Argument to pass to the method, in Candid textual format
    argument: Option<String>,
    /// Path to a candid file to analyze the argument
    #[clap(long, short)]
    candid: Option<String>,
    /// The type of input (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    in_type: helper::ArgType,
    /// The type of output (raw or idl).
    #[clap(long, short, possible_values = & (["raw", "idl"]), default_value = "idl")]
    out_type: helper::ArgType,
    /// An optional field to set the expiry time on requests. Can be a human
    /// readable time (like `100s`) or a number of seconds.
    #[clap(long)]
    ttl: Option<humantime::Duration>,
}

#[async_trait]
impl AsyncCommand for UpdateOpts {
    async fn async_exec(self, env: &mut Env) -> Result<()> {
        let canister_id =
            &Principal::from_text(self.canister_id).context("Invalid canister principal format")?;

        let method_name = &self.method_name;

        let argument = self.argument.as_deref();

        let in_type = &self.in_type;

        let out_type = &self.out_type;

        let method_type = match &self.candid {
            None => None,
            Some(path) => helper::get_candid_type(path, method_name)
                .context("Failed to get method type from candid file")?,
        };

        let arg_blob = helper::blob_from_arguments(argument, in_type, &method_type)
            .context("Invalid arguments")?;

        // 5 minutes is max ingress timeout
        let timeout = self
            .ttl
            .map(|ht| ht.into())
            .unwrap_or_else(|| Duration::from_secs(60 * 5));

        let effective_canister_id =
            utils::get_effective_canister_id(method_name, &arg_blob, canister_id)?;

        let agent = env.create_agent().await?;
        let result = agent
            .update(canister_id, method_name)
            .with_effective_canister_id(effective_canister_id)
            .with_arg(&arg_blob)
            .expire_after(timeout)
            .call_and_wait(helper::waiter_with_exponential_backoff())
            .await;

        match result {
            Ok(blob) => {
                helper::print_idl_blob(&blob, out_type, &method_type)
                    .context("Failed to print result blob")?;
            }
            Err(AgentError::TransportError(_)) => return Ok(()),
            Err(AgentError::HttpError(HttpErrorPayload {
                status,
                content_type,
                content,
            })) => {
                let mut error_message =
                    format!("Server returned an HTTP Error:\n  Code: {}\n", status);
                match content_type.as_deref() {
                    None => {
                        error_message.push_str(&format!(
                            "  Content:     {}\n",
                            String::from_utf8_lossy(&content)
                        ));
                    }
                    Some(x) => {
                        error_message.push_str(&format!("  ContentType: {}\n", x));
                        error_message.push_str(&format!(
                            "  Content:     {}\n",
                            String::from_utf8_lossy(&content)
                        ));
                    }
                }
                bail!(error_message);
            }
            Err(s) => Err(s).context("Got an error when make the canister call")?,
        }

        Ok(())
    }
}
