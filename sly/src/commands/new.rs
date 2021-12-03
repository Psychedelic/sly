use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use std::{fs, io};
use std::path::Path;
use clap::Parser as Clap;
use mkdirp::mkdirp;

#[derive(Clap)]
pub struct NewOpts {
    #[clap(long)]
    name: String,
    /// The template to use for the new project.
    #[clap(long, default_value = "simple")]
    template: String,
}

impl Command for NewOpts {
    fn exec(self, _: &mut Env) -> Result<(), anyhow::Error> {
        if self.template == String::from("fungible_token") {
            mkdirp(Path::new(&self.name));
            extract_from_zip(&String::from("../fungible_token.zip"));
        }
        Ok(())
    }
}

fn extract_from_zip(file_name: &String) {
    let fname = std::path::Path::new(file_name);
    println!("{}", fname.display());
    let file = fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}