#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use std::path::PathBuf;
use std::sync::LazyLock;

use log::error;

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
}
