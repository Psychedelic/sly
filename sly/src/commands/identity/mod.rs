use anyhow::Result;
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

pub mod create;
pub mod get_principal;
pub mod import;
pub mod list;
pub mod remove;
pub mod rename;
pub mod use_identity;
pub mod whoami;

#[derive(Clap)]
pub enum IdentitySubCommands {
    /// List the identities imported or created.
    List(list::IdentityListOpts),
    /// Create a new random identity.
    Create(create::IdentityCreateOpts),
    /// Import a new identity from a PEM file.
    Import(import::IdentityImportOpts),
    /// Set the default identity.
    Use(use_identity::IdentityUseOpts),
    /// Remove an identity and files associated with it.
    Remove(remove::IdentityRemoveOpts),
    /// Rename an identity.
    Rename(rename::IdentityRenameOpts),
    /// Print the current default identity.
    Whoami(whoami::IdentityWhoamiOpts),
    /// Print the principal id of the current identity.
    GetPrincipal(get_principal::IdentityGetPrincipalOpts),
}

impl Command for IdentitySubCommands {
    fn exec(self, env: &mut Env) -> Result<()> {
        match self {
            IdentitySubCommands::List(opts) => opts.exec(env),
            IdentitySubCommands::Create(opts) => opts.exec(env),
            IdentitySubCommands::Import(opts) => opts.exec(env),
            IdentitySubCommands::Use(opts) => opts.exec(env),
            IdentitySubCommands::Remove(opts) => opts.exec(env),
            IdentitySubCommands::Rename(opts) => opts.exec(env),
            IdentitySubCommands::Whoami(opts) => opts.exec(env),
            IdentitySubCommands::GetPrincipal(opts) => opts.exec(env),
        }
    }
}
