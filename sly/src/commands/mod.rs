use anyhow::Result;
use clap::Parser as Clap;
use std::path::PathBuf;

use crate::lib::command::{AsyncCommand, Command};
use crate::lib::env::Env;

mod build;
mod call;
mod candid;
mod create_canister;
mod deploy;
mod identity;
mod install_code;
mod new;
mod principal;
mod replica;
mod wasm;

/// Psychedelic's CLI for the Internet Computer.
#[derive(Clap)]
#[clap(version = "0.1", author = "Psychedelic Team")]
pub struct App {
    /// The network to use when making calls to the I.C.
    #[clap(short, long, default_value = "local")]
    pub network: String,
    /// The identity that should be used. This overwrites the
    /// default identity.
    #[clap(long)]
    pub identity: Option<String>,
    /// Optional path to the sly.json file.
    #[clap(long)]
    pub config: Option<PathBuf>,
    /// A level of verbosity, can be used multiple times.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    #[clap(subcommand)]
    pub sub: AppSubCommands,
}

#[derive(Clap)]
pub enum AppSubCommands {
    /// The commands to deal with Candid IDL files and values.
    #[clap(subcommand)]
    Candid(candid::CandidSubCommands),
    /// Set of commands to manage the identities used by this program.
    #[clap(subcommand)]
    Identity(identity::IdentitySubCommands),
    /// Set of commands to manage the local replica and run management methods.
    #[clap(subcommand)]
    Replica(replica::ReplicaSubCommands),
    /// Utilities to work with WASM files.
    #[clap(subcommand)]
    Wasm(wasm::WasmSubCommands),
    /// Create a new project.
    New(new::NewOpts),
    /// Install the code for the given canister(s).
    InstallCode(install_code::InstallOpts),
    /// Build the wasm for the given canister(s).
    Build(build::BuildOpts),
    /// Build the wasm for the given canister(s).
    CreateCanister(create_canister::CreateCanisterOpts),
    /// Deploy the canisters of the current workspace.
    Deploy(deploy::DeployOpts),
    /// Search for a given principal id.
    PrincipalGen(principal::PrincipalOpts),
    /// Canister call
    Call(call::CallSubCommands),
}

impl Command for AppSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            AppSubCommands::Candid(sub) => sub.exec(env),
            AppSubCommands::Identity(sub) => sub.exec(env),
            AppSubCommands::Replica(sub) => sub.exec(env),
            AppSubCommands::Wasm(sub) => sub.exec(env),
            AppSubCommands::New(opts) => opts.exec(env),
            AppSubCommands::InstallCode(opts) => opts.exec(env),
            AppSubCommands::Build(opts) => opts.exec(env),
            AppSubCommands::CreateCanister(opts) => opts.exec(env),
            AppSubCommands::Deploy(opts) => opts.exec(env),
            AppSubCommands::PrincipalGen(opts) => opts.exec(env),
            AppSubCommands::Call(opts) => opts.exec(env),
        }
    }
}
