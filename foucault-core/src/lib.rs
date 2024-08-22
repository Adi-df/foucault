pub mod api;
pub mod link_repr;
pub mod note_repr;
pub mod permissions;
pub mod pretty_error;
pub mod tag_repr;

pub use pretty_error::PrettyError;

use serde::{Deserialize, Serialize};

use crate::permissions::Permissions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookApiInfo {
    pub name: String,
    pub permissions: Permissions,
}
