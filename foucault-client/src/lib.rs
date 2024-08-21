#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::{path::PathBuf, sync::LazyLock};

use anyhow::Result;
use log::error;
use thiserror::Error;

use reqwest::Client;

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

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unable to connect to the remote endpoint : {0}")]
    UnableToConnect(reqwest::Error),
    #[error("Unable to ping notebook name : {0}")]
    UnableToPingName(reqwest::Error),
    #[error("Unable to contact the remote notebook : {0}")]
    UnableToContactRemoteNotebook(reqwest::Error),
    #[error("Unable to parse the request result : {0}")]
    UnableToParseResponse(reqwest::Error),
}

pub struct NotebookAPI {
    pub name: String,
    pub endpoint: String,
    pub client: Client,
}

impl NotebookAPI {
    pub async fn new(endpoint: String) -> Result<Self> {
        let name = reqwest::get(format!("{endpoint}/name"))
            .await
            .map_err(ApiError::UnableToConnect)?
            .text()
            .await
            .map_err(ApiError::UnableToPingName)?;

        Ok(Self {
            name,
            endpoint,
            client: Client::new(),
        })
    }

    #[must_use]
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.endpoint, path)
    }
}
