pub mod api;
pub mod link_repr;
pub mod note_repr;
pub mod pretty_error;
pub mod tag_repr;

use serde::{Deserialize, Serialize};

pub use pretty_error::PrettyError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Permissions {
    ReadWrite,
    ReadOnly,
}

impl Permissions {
    pub fn writtable(&self) -> bool {
        match self {
            Permissions::ReadWrite => true,
            Permissions::ReadOnly => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookApiInfo {
    pub name: String,
    pub permissions: Permissions,
}
