mod explore;
mod helpers;
mod logger;
mod note;
mod notebook;
mod notebook_selector;

use std::fs;
use std::path::PathBuf;

use crate::logger::SimpleLogger;
use anyhow::Result;
use log::{error, info};

use clap::{Parser, Subcommand};

use crate::explore::explore;
use crate::notebook::Notebook;
use crate::notebook_selector::open_selector;

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
    Delete { name: String },
}

fn main() -> Result<()> {
    log::set_boxed_logger(Box::new(SimpleLogger::new(true)))?;
    log::set_max_level(log::LevelFilter::Info);

    info!("Start foucault");

    let app_dir_path: PathBuf = {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("foucault")
        } else {
            error!("User data directory is unavailable.");
            unimplemented!();
        }
    };

    if !app_dir_path.exists() {
        if fs::create_dir(&app_dir_path).is_err() {
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
                info!("Create notebook {name}.");
                Notebook::new_notebook(name, &app_dir_path)?;
            }
            Commands::Open { name } => {
                info!("Open notebook {name}.");
                explore(&Notebook::open_notebook(name, &app_dir_path)?)?;
            }
            Commands::Delete { name } => {
                info!("Delete notebook {name}.");
                Notebook::delete_notebook(name, &app_dir_path)?;
            }
        }
    } else {
        info!("Open default notebook manager.");

        if let Some(name) = open_selector(&app_dir_path)? {
            info!("Open notebook selected : {name}.");
            explore(&Notebook::open_notebook(name.as_str(), &app_dir_path)?)?;
        }
    }

    Ok(())
}
