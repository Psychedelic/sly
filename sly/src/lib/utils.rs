use std::str::FromStr;

use anyhow::{bail, Context, Result};
use candid::{CandidType, Decode, Deserialize, Principal};
use ic_utils::interfaces::management_canister::{
    builders::{CanisterInstall, CanisterSettings},
    MgmtMethod,
};

/// Flatten a result, because .flatten() is unstable.
#[inline]
pub fn result_flatten<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
    match result {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e),
    }
}

/// Get effective canister id
pub fn get_effective_canister_id(
    method_name: &str,
    arg_value: &[u8],
    canister_id: &Principal,
) -> Result<Principal> {
    if Principal::management_canister() != *canister_id {
        return Ok(*canister_id);
    }
    let method_name = MgmtMethod::from_str(method_name).with_context(|| {
        format!(
            "Attempted to call an unsupported management canister method: {}",
            method_name
        )
    })?;
    match method_name {
        MgmtMethod::CreateCanister | MgmtMethod::RawRand => bail!(
            "{} can only be called via an inter-canister call.",
            method_name.as_ref()
        ),
        MgmtMethod::InstallCode => {
            let install_args = candid::Decode!(arg_value, CanisterInstall)
                .context("Argument is not valid for CanisterInstall")?;
            Ok(install_args.canister_id)
        }
        MgmtMethod::StartCanister
        | MgmtMethod::StopCanister
        | MgmtMethod::CanisterStatus
        | MgmtMethod::DeleteCanister
        | MgmtMethod::DepositCycles
        | MgmtMethod::UninstallCode
        | MgmtMethod::ProvisionalTopUpCanister => {
            #[derive(CandidType, Deserialize)]
            struct In {
                canister_id: Principal,
            }
            let in_args =
                candid::Decode!(arg_value, In).context("Argument is not a valid Principal")?;
            Ok(in_args.canister_id)
        }
        MgmtMethod::ProvisionalCreateCanisterWithCycles => Ok(Principal::management_canister()),
        MgmtMethod::UpdateSettings => {
            #[derive(CandidType, Deserialize)]
            struct In {
                canister_id: Principal,
                settings: CanisterSettings,
            }
            let in_args = candid::Decode!(arg_value, In)
                .context("Argument is not valid for UpdateSettings")?;
            Ok(in_args.canister_id)
        }
    }
}
