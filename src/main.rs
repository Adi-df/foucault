#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod notebook_selector;

use std::{env, sync::Arc};

use log::info;

use tokio::fs;

use clap::{crate_version, Parser, Subcommand};
use question::{Answer, Question};

use foucault_client::{explore::explore, NotebookAPI, APP_DIR_PATH};
use foucault_core::{permissions::Permissions, pretty_error, PrettyError};
use foucault_server::notebook::Notebook;

use crate::notebook_selector::open_selector;

pub const LOCAL_ADRESS: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 8078;

#[derive(Parser)]
#[command(
    author = "Adrien Degliame <adidf-web@laposte.net>",
    version = crate_version!(),
    about = "The Foucault notebook CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new notebook")]
    Create {
        #[arg(help = "The new notebook's name")]
        name: String,
        #[arg(short, long, help = "Create the notebook in the current directory")]
        local: bool,
    },
    #[command(about = "Open a notebook")]
    Open {
        #[arg(help = "The name of the notebook to open")]
        name: String,
        #[arg(
            short,
            long,
            help = "The internal port that should be used by foucault"
        )]
        port: Option<u16>,
    },
    #[command(about = "Serve a notebook for remote connection")]
    Serve {
        #[arg(help = "The name of the notebook to serve")]
        name: String,
        #[arg(short, long, help = "The port on which the notebook should be exposed")]
        port: Option<u16>,
        #[arg(long, help = "Only allow read only operations")]
        read_only: bool,
    },
    #[command(about = "Connect to a remote notebook")]
    Connect {
        #[arg(help = "The address at which the notebook is hosted")]
        endpoint: String,
    },
    #[command(about = "Delete a notebook")]
    Delete {
        #[arg(help = "The name of the notebook to delete")]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Start foucault");

    if !APP_DIR_PATH.exists() {
        if fs::create_dir(&*APP_DIR_PATH).await.is_err() {
            pretty_error!("Unable to create the app's directory.");
            todo!();
        }
    } else if !APP_DIR_PATH.is_dir() {
        pretty_error!("Another file already exists.");
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
                    .await
                    .pretty_unwrap();
                } else {
                    Notebook::new_notebook(name.trim(), &APP_DIR_PATH)
                        .await
                        .pretty_unwrap();
                };
                println!("Notebook {name} was successfully created.");
            }
            Commands::Open { name, port } => {
                info!("Open notebook {name}.");
                let notebook = Arc::new(
                    Notebook::open_notebook(name, &APP_DIR_PATH)
                        .await
                        .pretty_unwrap(),
                );
                let endpoint = format!("http://{LOCAL_ADRESS}:{}", port.unwrap_or(DEFAULT_PORT));
                tokio::spawn(foucault_server::serve(
                    notebook,
                    Permissions::ReadWrite,
                    port.unwrap_or(DEFAULT_PORT),
                ));
                let notebook_api = NotebookAPI::new(endpoint).await.pretty_unwrap();
                explore(&notebook_api).await.pretty_unwrap();
            }
            Commands::Connect { endpoint } => {
                info!("Connect to a notebook at address {endpoint}.");
                let notebook_api = NotebookAPI::new(endpoint.clone()).await.pretty_unwrap();
                explore(&notebook_api).await.pretty_unwrap();
            }
            Commands::Serve {
                name,
                read_only,
                port,
            } => {
                info!("Open notebook {name}.");
                let notebook = Arc::new(
                    Notebook::open_notebook(name, &APP_DIR_PATH)
                        .await
                        .pretty_unwrap(),
                );
                println!(
                    "Serving notebook {} at {LOCAL_ADRESS}:{}",
                    &notebook.name,
                    port.unwrap_or(DEFAULT_PORT)
                );
                foucault_server::serve(
                    notebook,
                    if *read_only {
                        Permissions::ReadOnly
                    } else {
                        Permissions::ReadWrite
                    },
                    port.unwrap_or(DEFAULT_PORT),
                )
                .await
                .pretty_unwrap();
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
                    Notebook::delete_notebook(name, &APP_DIR_PATH)
                        .await
                        .pretty_unwrap();
                } else {
                    println!("Cancel.");
                }
            }
        }
    } else {
        info!("Open the default notebook selector.");

        if let Some(name) = open_selector(&APP_DIR_PATH).pretty_unwrap() {
            info!("Open the notebook selected : {name}.");
            let notebook = Arc::new(
                Notebook::open_notebook(name.as_str(), &APP_DIR_PATH)
                    .await
                    .pretty_unwrap(),
            );
            let endpoint = format!("http://{LOCAL_ADRESS}:{DEFAULT_PORT}");
            tokio::spawn(foucault_server::serve(
                notebook,
                Permissions::ReadWrite,
                DEFAULT_PORT,
            ));
            let notebook_api = NotebookAPI::new(endpoint).await.pretty_unwrap();
            explore(&notebook_api).await.pretty_unwrap();
        }
    }
}
