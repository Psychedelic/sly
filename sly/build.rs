use std::fs;
use std::io::Write;
use std::include_bytes;
use zip::write::FileOptions;


fn main() {
    let template_paths = fs::read_dir("../templates/").unwrap();

    for template in template_paths {
        println!("Name: {}", template.unwrap().path().display())
    }
}
