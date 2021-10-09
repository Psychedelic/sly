use clap::Clap;
use crate::lib::command::Command;
use crate::lib::env::Env;
use std::path::PathBuf;
use anyhow::Context;
use crate::lib::private_key::PrivateKey;

#[derive(Clap)]
pub struct PrincipalOpts {
    patterns: Vec<String>,
    #[clap(long, short)]
    out_dir: String,
}

impl Command for PrincipalOpts {
    fn exec(self, _env: &mut Env) -> anyhow::Result<()> {
        let dir = PathBuf::from(&self.out_dir);
        std::fs::create_dir_all(&dir).context("Can not create the out directory.");

        println!("Searching for patterns {:?}", self.patterns);

        loop {
            let key = PrivateKey::generate();
            let principal = key.clone().into_identity().sender().unwrap().to_string();

            if self.patterns.iter().any(|x| principal.contains(x)) {
                let path = dir.join(format!("{}.pem", principal));
                println!("Found Principal {}", principal);
                key.store_pem_file("qti3e", path);
            }
        }
    }
}
