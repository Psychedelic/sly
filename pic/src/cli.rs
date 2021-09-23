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
        /// Path to the candid file.
        filename: String,
        /// If true the formatted version will be written to the input file, otherwise it'll
        /// be printed to the stdout.
        #[clap(short)]
        write: bool,
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

fn is_principal(text: &str) -> Result<(), String> {
    Principal::from_text(text)
        .map(|_| ())
        .map_err(|e| format!("Not a valid principal id."))
}
