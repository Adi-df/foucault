#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod explore;
mod helpers;
mod links;
mod markdown;
mod note;
mod states;
mod tag;

pub struct NotebookAPI {
    pub name: String,
    pub endpoint: &'static str,
}
