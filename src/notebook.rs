use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::error;
use thiserror::Error;

use rusqlite::Connection;

use crate::links::LinksTable;
use crate::note::NotesTable;
use crate::tag::{TagsJoinTable, TagsTable};

pub struct Notebook {
    pub name: String,
    file: PathBuf,
    database: Connection,
}

#[derive(Error, Debug)]
pub enum OpeningError {
    #[error("No notebook named {name:?} was found.")]
    NotebookNotFound { name: String },
}

#[derive(Error, Debug)]
pub enum CreationError {
    #[error("Another notebook named {name:?} was found.")]
    NotebookAlreadyExists { name: String },
}

#[derive(Error, Debug)]
pub enum SuppressionError {
    #[error("No notebook named {name:?} was found.")]
    NoNotebookExists { name: String },
}

impl Notebook {
    pub fn db(&self) -> &Connection {
        &self.database
    }

    pub fn dir(&self) -> Option<&Path> {
        self.file.parent()
    }

    pub fn open_notebook(name: &str, dir: &Path) -> Result<Self> {
        let notebook_path = dir.join(name);

        if !notebook_path.exists() {
            error!("The notebook \"{name}\" was not found.");
            return Err(OpeningError::NotebookNotFound {
                name: name.to_owned(),
            }
            .into());
        }

        let database = Connection::open(&notebook_path).unwrap_or_else(|_| {
            error!("Unable to open the notebook \"{name}\".");
            todo!();
        });

        Ok(Notebook {
            name: name.to_owned(),
            file: notebook_path,
            database,
        })
    }

    pub fn new_notebook(name: &str, dir: &Path) -> Result<Self> {
        let notebook_path = dir.join(name);

        if notebook_path.exists() {
            error!("A notebook named \"{name}\" already exists.");
            return Err(CreationError::NotebookAlreadyExists {
                name: name.to_owned(),
            }
            .into());
        }

        let database = Connection::open(&notebook_path).unwrap_or_else(|_| {
            error!("Unable to open the notebook \"{name}\".");
            todo!();
        });

        // Initialize
        NotesTable::create(&database)?;
        TagsTable::create(&database)?;
        TagsJoinTable::create(&database)?;
        LinksTable::create(&database)?;

        Ok(Notebook {
            name: name.to_owned(),
            file: notebook_path,
            database,
        })
    }

    pub fn delete_notebook(name: &str, dir: &Path) -> Result<()> {
        let notebook_path = dir.join(name);

        if !notebook_path.exists() {
            error!("No notebook named {name} exists.");
            return Err(SuppressionError::NoNotebookExists {
                name: name.to_owned(),
            }
            .into());
        }

        fs::remove_file(notebook_path)?;
        Ok(())
    }
}
