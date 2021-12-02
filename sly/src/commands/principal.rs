use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;

use anyhow::{anyhow, Context};
use clap::Parser as Clap;

use crate::lib::command::Command;
use crate::lib::env::Env;
use crate::lib::private_key::PrivateKey;

#[derive(Clap)]
pub struct PrincipalOpts {
    patterns: Vec<String>,
    #[clap(long, short)]
    out_dir: String,
    #[clap(long)]
    num_threads: Option<usize>,
}

impl Command for PrincipalOpts {
    fn exec(self, _env: &mut Env) -> anyhow::Result<()> {
        let dir = PathBuf::from(&self.out_dir);
        std::fs::create_dir_all(&dir).context("Can not create the out directory.")?;

        let num_threads = self.num_threads.unwrap_or_else(num_cpus::get);

        println!(
            "Searching for patterns {:?} using {} thread(s)",
            self.patterns, num_threads
        );

        let handles = (0..num_threads)
            .into_iter()
            .map(|_| run_thread(dir.clone(), self.patterns.clone()))
            .collect::<Vec<JoinHandle<anyhow::Result<()>>>>();

        for handle in handles {
            if handle.join().is_err() {
                return Err(anyhow!("Couldn't join on the associated thread"));
            }
        }

        Ok(())
    }
}

fn run_thread(dir: PathBuf, patterns: Vec<String>) -> JoinHandle<anyhow::Result<()>> {
    log::info!("Starting thread.");

    thread::spawn(move || loop {
        let key = PrivateKey::generate();
        let principal = key.clone().into_identity().sender().unwrap().to_string();

        if patterns.iter().any(|x| principal.contains(x)) {
            let path = dir.join(format!("{}.pem", principal));
            println!("Found Principal {}", principal);
            key.store_pem_file("", path)?;
        }
    })
}
