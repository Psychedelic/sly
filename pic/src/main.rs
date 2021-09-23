mod candid;
mod cli;
mod identity;
mod replica;
mod wasm;
mod workspace;

use clap::{App, Clap};
use human_panic::setup_panic;
use log::{error, info, trace, warn, LevelFilter};
use pretty_env_logger::formatted_builder;

fn main() {
    setup_panic!();

    formatted_builder().filter(None, LevelFilter::Trace).init();

    let matches: cli::App = cli::App::parse();
}
