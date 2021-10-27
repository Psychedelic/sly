use anyhow::Result;
use clap::Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;

#[derive(Clap)]
pub struct NewOpts {
    /// The template to use for the new project.
    #[clap(long, default_value = "simple")]
    template: String,
}

impl Command for NewOpts {
    fn exec(self, _: &mut Env) -> Result<()> {
        todo!()
    }
}
