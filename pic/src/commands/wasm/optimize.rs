use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct WasmOptimizeOpts {
    /// Path to the WASM file.
    filename: String,
    /// Where to write the optimized WASM.
    #[clap(short, long)]
    output: String,
}

impl Command for WasmOptimizeOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        todo!()
    }
}
