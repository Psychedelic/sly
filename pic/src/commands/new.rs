use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Clap;

#[derive(Clap)]
pub struct NewOpts {
    /// The template to use for the new project.
    #[clap(long, default_value = "simple")]
    template: String,
}

impl Command for NewOpts {
    fn exec(self, _: &Env) -> Result<()> {
        todo!()
    }
}
