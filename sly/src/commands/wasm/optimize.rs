use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Clap;
use humansize::{file_size_opts, FileSize};

use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::wasm_optimizer;

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
        let passes = wasm_optimizer::create();
        let mut wasm = fs::read(self.filename).context("Can not read the input file.")?;
        let original_size = wasm.len();

        for pass in passes {
            log::info!("Running optimization pass {}", pass.short_name());

            let original_size = wasm.len();
            wasm = pass.opt(&wasm).context("Optimization pass failed.")?;
            let size = wasm.len();

            log::info!(
                "    Size:          {:>8} ({:3.1}% smaller)",
                size.file_size(file_size_opts::BINARY).unwrap(),
                (1.0 - ((size as f64) / (original_size as f64))) * 100.0
            );
        }

        let size = wasm.len();
        log::info!(
            "Final Size:        {:>8} ({:3.1}% smaller)",
            size.file_size(file_size_opts::BINARY).unwrap(),
            (1.0 - ((size as f64) / (original_size as f64))) * 100.0
        );

        let path = PathBuf::from(&self.output);
        if let Some(parent_path) = path.parent() {
            log::trace!("Creating output directory {:?}", parent_path);
            fs::create_dir_all(parent_path)
                .context("Can not create the parent directory for the output.")?;
        }

        log::trace!("Writing the optimized wasm to {:?}", path);
        fs::write(path, wasm).context(format!(
            "Failed to write the optimized wasm to {}.",
            self.output
        ))?;

        Ok(())
    }
}
