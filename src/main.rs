use std::fs::{self, ReadDir};
use std::io;
use std::path::PathBuf;

use dirs;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    author = "Adrien Degliame <adidf-web@laposte.net>",
    version = "0.0.0",
    about = "The Foucault notebook CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Create { name: String },
    Open { name: String },
}

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

    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::Create { name } => println!("Create notebook {name}"),
            Commands::Open { name } => println!("Open notebook {name}"),
        }
    } else {
        println!("Open default notebook manager");
    }
}
