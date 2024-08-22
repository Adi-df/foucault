use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: Arc<String>,
    pub color: u32,
}

#[derive(Debug, Clone, Copy, Error, Serialize, Deserialize)]
pub enum TagError {
    #[error("A similarly named tag already exists.")]
    AlreadyExists,
    #[error("The provided tag name is empty.")]
    EmptyName,
    #[error("No such tag exists.")]
    DoesNotExists,
}
