use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::{AppSettings, Clap};

mod call;
mod candid;
mod deploy;
mod identity;
mod new;
mod principal;
mod replica;
mod wasm;

/// Psychedelic's CLI for the Internet Computer.
#[derive(Clap)]
#[clap(version = "0.1", author = "Psychedelic Team")]
#[clap(global_setting = AppSettings::ColoredHelp)]
pub struct App {
    /// The network to use when making calls to the I.C.
    #[clap(short, long, possible_values=&(["local","ic"]), default_value = "local")]
    pub network: String,
    #[clap(long)]
    pub identity: Option<String>,
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
    /// Deploy the canisters of the current workspace.
    Deploy(deploy::DeployOpts),
    /// Search for a given principal id.
    PrincipalGen(principal::PrincipalOpts),
}

impl Command for AppSubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            AppSubCommands::Candid(sub) => sub.exec(env),
            AppSubCommands::Identity(sub) => sub.exec(env),
            AppSubCommands::Replica(sub) => sub.exec(env),
            AppSubCommands::Wasm(sub) => sub.exec(env),
            AppSubCommands::New(opts) => opts.exec(env),
            AppSubCommands::Deploy(opts) => opts.exec(env),
            AppSubCommands::PrincipalGen(opts) => opts.exec(env),
        }
    }
}
