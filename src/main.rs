use std::fs::{self, ReadDir};
use std::io;
use std::path::PathBuf;

use log::{error, trace};

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
            error!("User data directory is unavailable");
            unimplemented!();
        }
    };

    let app_dir: ReadDir = fs::read_dir(&app_dir_path).unwrap_or_else(|err| {
        if matches!(err.kind(), io::ErrorKind::NotFound) {
            if fs::create_dir(&app_dir_path).is_err() {
                error!("Unable to create the app directory");
            }
            fs::read_dir(&app_dir_path).unwrap_or_else(|err| {
                error!("Couldn't read the just created app directory. Should be unreachable.");
                panic!("{}", err.to_string());
            })
        } else {
            error!("Unknown error occured while opening the app directory");
            panic!("{}", err.to_string());
        }
    });

    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::Create { name } => trace!("Create notebook {name}"),
            Commands::Open { name } => trace!("Open notebook {name}"),
        }
    } else {
        trace!("Open default notebook manager");
    }
}
