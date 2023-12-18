mod notebook;

use std::fs;
use std::path::PathBuf;

use log::{error, trace};

use dirs;

use clap::{Parser, Subcommand};

use crate::notebook::Notebook;

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
            error!("User data directory is unavailable.");
            unimplemented!();
        }
    };

    if !app_dir_path.exists() {
        if let Err(_) = fs::create_dir(&app_dir_path) {
            error!("Unable to create app directory.");
            todo!();
        }
    } else if !app_dir_path.is_dir() {
        error!("Another file already exists.");
        todo!();
    }

    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::Create { name } => {
                trace!("Create notebook {name}.");
            }
            Commands::Open { name } => {
                Notebook::open_notebook(&name, &app_dir_path).unwrap();
                trace!("Open notebook {name}.")
            }
        }
    } else {
        trace!("Open default notebook manager.");
    }
}
