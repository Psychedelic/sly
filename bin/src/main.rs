pub mod candid;
pub mod core;
pub mod identity;

use clap::{load_yaml, App};
use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(
    version = "0.1",
    author = "Psychedelic Team",
    about = "Psychedelic's CLI for the Internet Computer."
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "0.1")]
    Candid(candid::Candid),
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Candid(c) => c.perform(),
    }
}

// pic candid check file.did
// pic candid fmt file.did --write
// pic candid install [canister-id] file.did

// pic identity list
// pic identity create
// pic identity import
// pic identity remove
// pic identity rename
// pic identity get-principal

// pic new --template X

// pic test
// pic build
// pic deploy
// pic deploy --mode upgrade

// pic canister create
// pic canister status
// pic canister delete
// pic canister call
// pic canister install
// pic canister test

// pic wasm run [canister]
// pic wasm optimize [canister]

// pic emulator
