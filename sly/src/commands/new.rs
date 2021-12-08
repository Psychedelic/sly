use crate::lib::command::Command;
use crate::lib::env::Env;
use anyhow::Result;
use clap::Parser as Clap;
use std::path::PathBuf;
use std::{fs, io};

const FUNGIBLE_TOKEN_TEMPLATE: &[u8] = include_bytes!("../../../target/assets/fungible_token.zip");
const NON_FUNGIBLE_TOKEN_TEMPLATE: &[u8] =
    include_bytes!("../../../target/assets/non_fungible_token.zip");
const RUST_BACKEND: &[u8] = include_bytes!("../../../target/assets/rust_backend.zip");

#[derive(Clap)]
pub struct NewOpts {
    /// The template to use for the new project.
    #[clap(long, default_value = "rust_backend")]
    template: String,
    /// Name of the new project.
    name: String,
}

impl Command for NewOpts {
    fn exec(self, _: &mut Env) -> Result<(), anyhow::Error> {
        if self.template == *"fungible_token" {
            let reader = std::io::Cursor::new(FUNGIBLE_TOKEN_TEMPLATE);
            let destination = PathBuf::from(self.name);
            extract_from_zip(destination, reader);
        } else if self.template == *"none_fungible_token" {
            let reader = std::io::Cursor::new(NON_FUNGIBLE_TOKEN_TEMPLATE);
            let destination = PathBuf::from(self.name);
            extract_from_zip(destination, reader);
        } else if self.template == *"rust_backend" {
            let reader = std::io::Cursor::new(RUST_BACKEND);
            let destination = PathBuf::from(self.name);
            extract_from_zip(destination, reader);
        }
        Ok(())
    }
}

fn extract_from_zip<R: std::io::Read + std::io::Seek>(destination: PathBuf, reader: R) {
    let mut archive = zip::ZipArchive::new(reader).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => destination.join(path),
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
