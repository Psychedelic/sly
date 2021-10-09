use clap::Clap;
use crate::lib::command::Command;
use crate::lib::env::Env;
use std::path::PathBuf;
use anyhow::Context;
use crate::lib::private_key::PrivateKey;
use std::thread;
use std::thread::JoinHandle;

#[derive(Clap)]
pub struct PrincipalOpts {
    patterns: Vec<String>,
    #[clap(long, short)]
    out_dir: String,
    #[clap(long)]
    num_threads: Option<usize>
}

impl Command for PrincipalOpts {
    fn exec(self, _env: &mut Env) -> anyhow::Result<()> {
        let dir = PathBuf::from(&self.out_dir);
        std::fs::create_dir_all(&dir).context("Can not create the out directory.")?;

        let num_threads = self.num_threads.unwrap_or(num_cpus::get());

        println!("Searching for patterns {:?} using {} thread(s)", self.patterns, num_threads);

        let handles = (0..num_threads).into_iter().map(|_| {
            run_thread(dir.clone(), self.patterns.clone())
        }).collect::<Vec<JoinHandle<()>>>();

        for handle in handles {
            handle.join();
        }

        Ok(())
    }
}

fn run_thread(dir: PathBuf, patterns: Vec<String>) -> JoinHandle<()> {
    log::info!("Starting thread.");

    thread::spawn(move || {
        loop {
            let key = PrivateKey::generate();
            let principal = key.clone().into_identity().sender().unwrap().to_string();

            if patterns.iter().any(|x| principal.contains(x)) {
                let path = dir.join(format!("{}.pem", principal));
                println!("Found Principal {}", principal);
                key.store_pem_file("qti3e", path);
            }
        }
    })
}