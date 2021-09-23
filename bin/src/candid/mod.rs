pub mod check;
pub mod fmt;

use clap::Clap;

#[derive(Clap)]
#[clap(about = "Utility commands to interact with the candid files.")]
pub struct Candid {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Type check a candid IDL file.")]
    Check(check::Check),
    #[clap(about = "Pretty print a candid IDL file.")]
    Fmt(fmt::Format),
}

impl Candid {
    pub fn perform(self) {
        match self.subcommand {
            SubCommand::Check(c) => c.perform(),
            SubCommand::Fmt(c) => c.perform(),
        }
    }
}
