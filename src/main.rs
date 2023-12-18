use dirs;
use std::fs::{self, ReadDir};
use std::io;
use std::path::PathBuf;

fn main() {
    let app_dir_path: PathBuf = {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("foucault")
        } else {
            unimplemented!();
        }
    };

    let app_dir: ReadDir = match fs::read_dir(&app_dir_path) {
        Ok(dir) => dir,
        Err(err) if matches!(err.kind(), io::ErrorKind::NotFound) => {
            fs::create_dir(&app_dir_path).unwrap();
            fs::read_dir(&app_dir_path).unwrap()
        }
        _ => unimplemented!(),
    };

    println!("{:?}", app_dir);
}
