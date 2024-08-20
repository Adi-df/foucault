#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::path::PathBuf;
use std::sync::LazyLock;

use log::error;

use reqwest::Client;

use foucault_server::notebook::Notebook;

pub mod explore;
mod helpers;
mod links;
mod markdown;
mod note;
mod states;
mod tag;

pub static APP_DIR_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("foucault")
    } else {
        error!("User data directory is unavailable.");
        unimplemented!();
    }
});

pub struct NotebookAPI {
    pub name: String,
    pub endpoint: &'static str,
    pub client: Client,
}

impl NotebookAPI {
    #[must_use]
    pub fn new(notebook: &Notebook) -> Self {
        Self {
            name: notebook.name.clone(),
            endpoint: "http://0.0.0.0:8078",
            client: Client::new(),
        }
    }

    #[must_use]
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.endpoint, path)
    }
}
