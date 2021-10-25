use crate::commands::call::helper;
use crate::commands::call::{Args, CallSubCommands};
use crate::lib::env::Env;
use crate::lib::utils;
use anyhow::{Context, Result};
use candid::Principal;
use std::ops::Deref;
use std::time::Duration;

pub async fn async_exec(args: &Args, opts: &CallSubCommands, env: &Env) -> Result<()> {
    let canister_id = &Principal::from_text(args.canister_id.deref())
        .context("Invalid canister principal format")?;

    let method_name = args.method_name.deref();

    let method_type = match &opts.candid {
        None => None,
        Some(path) => helper::get_candid_type(path, method_name)
            .context("Failed to get method type from candid file")?,
    };

    let arg_blob =
        helper::blob_from_arguments(args.argument.as_deref(), &opts.in_type, &method_type)
            .context("Invalid arguments")?;

    // 5 minutes is max ingress timeout
    let timeout = opts
        .ttl
        .map(|ht| ht.into())
        .unwrap_or_else(|| Duration::from_secs(60 * 5));

    let effective_canister_id =
        utils::get_effective_canister_id(method_name, &arg_blob, canister_id)
            .context("Failed to get effective_canister_id for this call")?;

    let agent = env.create_agent().await?;
    let result = agent
        .query(canister_id, method_name)
        .with_effective_canister_id(effective_canister_id)
        .with_arg(&arg_blob)
        .expire_after(timeout)
        .call()
        .await;

    helper::print_agent_result(result, &opts.out_type, &method_type)
}
