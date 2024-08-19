#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![warn(unused_crate_dependencies)]
pub mod explore;
mod helpers;
mod links;
mod markdown;
mod note;
pub mod notebook;
mod states;
mod tag;
