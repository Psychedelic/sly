use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;

fn process_template_directory(src_dir: &str, dst_file: &str) {
    zip_directory(src_dir, dst_file, zip::CompressionMethod::Stored)
        .unwrap_or_else(|_| panic!("Failed to create zip file for '{}'", src_dir));
    println!("done: {} written to {}", src_dir, dst_file);
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn zip_directory(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}

fn main() {
    fs::create_dir_all("../target/assets").expect("Failed to create the target/assets/ directory.");
    process_template_directory(
        "../templates/fungible_token",
        "../target/assets/fungible_token.zip",
    );
    process_template_directory(
        "../templates/non_fungible_token",
        "../target/assets/non_fungible_token.zip",
    );
    process_template_directory(
        "../templates/rust_backend",
        "../target/assets/rust_backend.zip",
    );

    /* let template_paths = fs::read_dir("../templates").unwrap();
    for template in template_paths {
        println!("Name: {}", template.unwrap().path().display());
        let template_dir = template.unwrap().path().to_str().unwrap();
        process_template_directory(template_dir, "../target/assets/fungible_token.zip");
    } */
}
