#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::{fmt::Display, path::PathBuf, sync::LazyLock};

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

pub trait PrettyError {
    type Item;
    fn pretty_unwrap(self) -> Self::Item;
}

impl<T, E> PrettyError for Result<T, E>
where
    E: Display,
{
    type Item = T;
    fn pretty_unwrap(self) -> Self::Item {
        match self {
            Ok(val) => val,
            Err(err) => {
                eprintln!("error : {err}");
                todo!();
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Unable to connect to the remote endpoint : {0}")]
    UnableToConnect(reqwest::Error),
    #[error("Unable to ping notebook name : {0}")]
    UnableToPingName(reqwest::Error),
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
            .map_err(|err| ApiError::UnableToConnect(err))?
            .text()
            .await
            .map_err(|err| ApiError::UnableToPingName(err))?;

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
