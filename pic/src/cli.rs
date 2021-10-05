use anyhow::Result;
use candid::Principal;
use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "0.1", author = "Psychedelic Team")]
#[clap(global_setting = AppSettings::ColoredHelp)]
/// Psychedelic's CLI for the Internet Computer.
pub struct App {
    /// The network to use when making calls to the I.C.
    #[clap(short, long, possible_values=&(["local","ic"]), default_value = "local")]
    pub network: String,
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
    Candid(CandidSubCommands),
    /// Set of commands to manage the identities used by this program.
    #[clap(subcommand)]
    Identity(IdentitySubCommands),
    /// Set of commands to manage the local replica and run management methods.
    #[clap(subcommand)]
    Replica(ReplicaSubCommands),
    /// Utilities to work with WASM files.
    #[clap(subcommand)]
    Wasm(WasmSubCommands),
    /// Create a new project.
    New {
        /// The template to use for the new project.
        #[clap(long, default_value = "simple")]
        template: String,
    },
    /// Deploy the canisters of the current workspace.
    Deploy {
        /// The canister to deploy.
        canister: Option<String>,
        /// The installation mode.
        #[clap(short, long, possible_values=&(["install","reinstall","upgrade"]), default_value = "install")]
        mode: String,
    },
}

#[derive(Clap)]
pub enum CandidSubCommands {
    /// Run the type verifier on a candid file.
    Check {
        /// Path to the candid file.
        filename: String,
    },
    /// Format a candid file.
    Format {
        /// Path to the candid files to format.
        files: Vec<String>,
    },
    /// Generate codes for the Candid files.
    Gen {
        /// Path to the candid files.
        files: Vec<String>,
        #[clap(short, long)]
        /// Directory
        out_dir: String,
        /// Generate JavaScript language bindings.
        #[clap(long)]
        js: bool,
        /// Generate TypeScript language bindings.
        #[clap(long)]
        ts: bool,
        /// Generate Motoko language bindings.
        #[clap(long)]
        motoko: bool,
    },
}

#[derive(Clap)]
pub enum IdentitySubCommands {
    /// List the identities imported or created.
    List {},
    /// Create a new random identity.
    Create {
        /// Name of the new generated identity.
        name: String,
    },
    /// Import a new identity from a PEM file.
    Import {
        /// Name of the new generated identity.
        name: String,
        /// Path to the PEM file.
        pem: String,
    },
    /// Set the default identity.
    Use {
        /// Name of the identity to be used as default.
        name: String,
    },
    /// Remove an identity and files associated with it.
    Remove {
        /// Name of the identity to be removed.
        name: String,
    },
    /// Rename an identity.
    Rename {
        /// Original name of the identity you want to change.
        name: String,
        /// The name for the given identity.
        new_name: String,
    },
    /// Print the current default identity.
    Whoami {},
    /// Print the principal id of the current identity.
    GetPrincipal {},
}

#[derive(Clap)]
pub enum ReplicaSubCommands {
    /// Start the local instance of the replica.
    Start {},
    /// Stop the local instance of the replica.
    Stop {},
    /// Print information about the running replica.
    Info {},
    /// Create a new canister.
    CreateCanister {
        /// The number of cycles to use for creating this canister.
        #[clap(short, long)]
        cycles: u64,
    },
    /// Install the given wasm on the canister.
    Install {
        /// The installation mode.
        #[clap(short, long, possible_values=&(["install","reinstall","upgrade"]), default_value = "install")]
        mode: String,
        /// Principal ID of the canister you want to install the WASM on.
        #[clap(validator = is_principal)]
        canister: String,
        /// Path to the WASM file.
        wasm: String,
    },
    /// Remove a canister from the subnet.
    DeleteCanister {
        /// Principal ID of the canister you want to remove.
        #[clap(validator = is_principal)]
        canister: String,
    },
    /// Obtain the status of a canister.
    CanisterStatus {
        /// Principal ID of the canister.
        #[clap(validator = is_principal)]
        canister: String,
    },
    /// Add a controller to the canister.
    AddController {
        /// Principal ID of the canister.
        #[clap(validator = is_principal)]
        canister: String,
        /// The new controller to be added.
        #[clap(validator = is_principal)]
        controller: String,
    },
}

#[derive(Clap)]
pub enum WasmSubCommands {
    /// Run a WASM file.
    Run {},
    /// Run an optimizer on a WASM binary.
    Optimize {
        /// Path to the WASM file.
        filename: String,
        /// Where to write the optimized WASM.
        #[clap(short, long)]
        output: String,
    },
}

fn is_principal(text: &str) -> Result<(), String> {
    Principal::from_text(text)
        .map(|_| ())
        .map_err(|_| format!("Not a valid principal id."))
}

impl App {
    pub fn run(&self) -> Result<()> {
        match &self.sub {
            AppSubCommands::Candid(s) => s.run(self),
            AppSubCommands::Identity(s) => s.run(self),
            AppSubCommands::Replica(s) => s.run(self),
            AppSubCommands::Wasm(s) => s.run(self),
            AppSubCommands::New { template } => crate::workspace::new::run(template.as_str()),
            AppSubCommands::Deploy { canister, mode } => {
                let canister = canister.clone();
                crate::workspace::deploy::run(canister, mode.as_str())
            }
        }
    }
}

impl CandidSubCommands {
    pub fn run(&self, app: &App) -> Result<()> {
        use crate::candid::gen::Languages;

        match self {
            CandidSubCommands::Check { filename } => crate::candid::check::run(filename.as_str()),
            CandidSubCommands::Format { files } => crate::candid::format::run(files),
            CandidSubCommands::Gen {
                files,
                out_dir,
                js,
                ts,
                motoko,
            } => crate::candid::gen::run(
                files,
                out_dir,
                Languages {
                    javascript: *js,
                    typescript: *ts,
                    motoko: *motoko,
                },
            ),
        }
    }
}

impl IdentitySubCommands {
    pub fn run(&self, app: &App) -> Result<()> {
        use crate::identity;

        match self {
            IdentitySubCommands::List {} => identity::list::run(),
            IdentitySubCommands::Create { name } => identity::create::run(name.as_str()),
            IdentitySubCommands::Import { name, pem } => {
                identity::import::run(name.as_str(), pem.as_str())
            }
            IdentitySubCommands::Use { name } => identity::default::run(name.as_str()),
            IdentitySubCommands::Remove { name } => identity::remove::run(name.as_str()),
            IdentitySubCommands::Rename { name, new_name } => identity::rename::run(name, new_name),
            IdentitySubCommands::Whoami {} => identity::whoami::run(),
            IdentitySubCommands::GetPrincipal {} => identity::get_principal::run(),
        }
    }
}

impl ReplicaSubCommands {
    pub fn run(&self, app: &App) -> Result<()> {
        match self {
            ReplicaSubCommands::Start { .. } => todo!(),
            ReplicaSubCommands::Stop { .. } => todo!(),
            ReplicaSubCommands::Info { .. } => todo!(),
            ReplicaSubCommands::CreateCanister { .. } => todo!(),
            ReplicaSubCommands::Install { .. } => todo!(),
            ReplicaSubCommands::DeleteCanister { .. } => todo!(),
            ReplicaSubCommands::CanisterStatus { .. } => todo!(),
            ReplicaSubCommands::AddController { .. } => todo!(),
        }
    }
}

impl WasmSubCommands {
    pub fn run(&self, app: &App) -> Result<()> {
        match self {
            WasmSubCommands::Run { .. } => todo!(),
            WasmSubCommands::Optimize { .. } => todo!(),
        }
    }
}
