use std::sync::Arc;

use thiserror::Error;

use serde::{Deserialize, Serialize};

use crate::tag_repr::Tag;

#[derive(Debug, Clone, Copy, Error, Serialize, Deserialize)]
pub enum NoteError {
    #[error("No such note exists.")]
    DoesNotExist,
    #[error("A similarly named note already exists.")]
    AlreadyExists,
    #[error("The provided note name is empty.")]
    EmptyName,
    #[error("The note already has the provided tag.")]
    NoteAlreadyTagged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub name: Arc<String>,
    pub content: Arc<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: i64,
    pub name: Arc<String>,
    pub tags: Vec<Tag>,
}
