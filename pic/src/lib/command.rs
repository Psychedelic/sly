use crate::lib::env::Env;
use anyhow::Result;

pub trait Command {
    /// Execute the command.
    fn exec(self, env: &mut Env) -> Result<()>;
}
