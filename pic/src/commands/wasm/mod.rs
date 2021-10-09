use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

pub mod optimize;

#[derive(Clap)]
pub enum WasmSubCommands {
    /// Run an optimizer on a WASM binary.
    Optimize(optimize::WasmOptimizeOpts),
}

impl Command for WasmSubCommands {
    fn exec(self, env: &Env) -> Result<()> {
        match self {
            WasmSubCommands::Optimize(opts) => opts.exec(env),
        }
    }
}
