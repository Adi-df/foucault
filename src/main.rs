#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod notebook_selector;

use std::env;
use std::sync::Arc;

use anyhow::Result;
use log::{error, info};

use tokio::fs;

use clap::{Parser, Subcommand};
use question::{Answer, Question};

use foucault_client::explore::explore;
use foucault_client::{NotebookAPI, APP_DIR_PATH};
use foucault_server::notebook::Notebook;

use crate::notebook_selector::open_selector;

#[derive(Parser)]
#[command(
    author = "Adrien Degliame <adidf-web@laposte.net>",
    version = "0.2.1",
    about = "The Foucault notebook CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Create {
        name: String,
        #[arg(short, long)]
        local: bool,
    },
    Open {
        name: String,
    },
    Delete {
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("Start foucault");

    if !APP_DIR_PATH.exists() {
        if fs::create_dir(&*APP_DIR_PATH).await.is_err() {
            error!("Unable to create app directory.");
            todo!();
        }
    } else if !APP_DIR_PATH.is_dir() {
        error!("Another file already exists.");
        todo!();
    }

    let cli = Cli::parse();

    if let Some(command) = &cli.command {
        match command {
            Commands::Create { name, local } => {
                info!("Create notebook {name}.");
                if *local {
                    Notebook::new_notebook(
                        name.trim(),
                        &env::current_dir().expect("The current directory isn't accessible"),
                    )
                    .await?;
                } else {
                    Notebook::new_notebook(name.trim(), &APP_DIR_PATH).await?;
                };
                println!("Notebook {name} was successfully created.");
            }
            Commands::Open { name } => {
                info!("Open notebook {name}.");
                let notebook = Arc::new(Notebook::open_notebook(name, &APP_DIR_PATH).await?);
                let notebook_api = NotebookAPI {
                    name: notebook.name.clone(),
                    endpoint: todo!(),
                };
                tokio::spawn(foucault_server::serve(notebook));
                explore(&notebook_api).await?;
            }
            Commands::Delete { name } => {
                info!("Delete notebook {name}.");
                if matches!(
                    Question::new(&format!(
                        "Are you sure you want to delete notebook {name} ?",
                    ))
                    .default(Answer::NO)
                    .show_defaults()
                    .confirm(),
                    Answer::YES
                ) {
                    println!("Proceed.");
                    Notebook::delete_notebook(name, &APP_DIR_PATH).await?;
                } else {
                    println!("Cancel.");
                }
            }
        }
    } else {
        info!("Open default notebook manager.");

        if let Some(name) = open_selector(&APP_DIR_PATH)? {
            info!("Open notebook selected : {name}.");
            let notebook = Arc::new(Notebook::open_notebook(name.as_str(), &APP_DIR_PATH).await?);
            let notebook_api = NotebookAPI {
                name: notebook.name.clone(),
                endpoint: todo!(),
            };
            tokio::spawn(foucault_server::serve(notebook));
            explore(&notebook_api).await?;
        }
    }

    Ok(())
}
