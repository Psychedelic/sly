mod actors;
mod candid;
mod cli;
mod identity;
mod lib;
mod replica;
mod wasm;
mod workspace;

use anyhow::Result;
use clap::Clap;
use human_panic::setup_panic;
use log::LevelFilter;
use pretty_env_logger::formatted_builder;

fn main() -> Result<()> {
    setup_panic!();

    let opts: cli::App = cli::App::parse();

    let level = if opts.verbose >= 3 {
        LevelFilter::Trace
    } else if opts.verbose == 2 {
        LevelFilter::Debug
    } else if opts.verbose == 1 {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    };

    formatted_builder().filter(None, level).init();
    opts.run()
}
