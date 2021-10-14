use crate::wasm_optimizer::OptimizationPass;
use anyhow::Result;
use wabt::{wasm2wat, wat2wasm};

pub struct RemoveDebugSymbolsPass {}

impl OptimizationPass for RemoveDebugSymbolsPass {
    fn short_name(&self) -> String {
        String::from("strip_data")
    }

    fn description(&self) -> String {
        String::from("Stripping Unused Data Segments")
    }

    fn opt(&self, wasm: &[u8]) -> Result<Vec<u8>> {
        let wat = wasm2wat(&wasm)?;
        Ok(wat2wasm(&wat)?)
    }
}
