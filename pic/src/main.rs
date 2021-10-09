use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;
use human_panic::setup_panic;
use log::LevelFilter;
use pretty_env_logger::formatted_builder;

mod actors;
mod commands;
mod lib;

fn main() -> Result<()> {
    setup_panic!();

    let opts: commands::App = commands::App::parse();

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

    let network = Some(opts.network.as_str());
    let identity = opts.identity.as_ref().map(|x| x.as_str());
    let env = Env::new(network, identity)?;

    opts.sub.exec(&env)
}
