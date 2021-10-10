use anyhow::Result;

mod binaryen;
mod strip_data;

pub trait OptimizationPass {
    fn short_name(&self) -> String;
    fn description(&self) -> String;
    fn opt(&self, wasm: &[u8]) -> Result<Vec<u8>>;
}

/// Return a vector of optimization passes.
pub fn create() -> Vec<Box<dyn OptimizationPass>> {
    vec![
        Box::new(strip_data::RemoveDebugSymbolsPass {}),
        Box::new(binaryen::BinaryenPass {}),
    ]
}
