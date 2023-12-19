use std::path::Path;

use anyhow::Result;
use log::error;
use thiserror::Error;

use rusqlite::Connection;
use sea_query::{ColumnDef, Iden, SqliteQueryBuilder, Table};

use crate::note::NoteCharacters;

pub struct Notebook {
    name: String,
    database: Connection,
}

#[derive(Error, Debug)]
pub enum OpeningError {
    #[error("No notebook named \"{name:?}\" was found.")]
    NotebookNotFound { name: String },
}

#[derive(Error, Debug)]
pub enum CreationError {
    #[error("Another notebook named \"{name:?}\" was found.")]
    NotebookAlreadyExists { name: String },
}

#[derive(Iden)]
pub struct NoteTable;

impl Notebook {
    pub fn open_notebook(name: &str, dir: &Path) -> Result<Self> {
        let notebook_path = dir.join(name);

        if !notebook_path.exists() {
            error!("The notebook \"{name}\" was not found.");
            return Err(OpeningError::NotebookNotFound {
                name: name.to_owned(),
            }
            .into());
        }

        let database = Connection::open(notebook_path).unwrap_or_else(|_| {
            error!("Unable to open the notebook \"{name}\".");
            todo!();
        });

        Ok(Notebook {
            name: name.to_owned(),
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

        let database = Connection::open(notebook_path).unwrap_or_else(|_| {
            error!("Unable to open the notebook \"{name}\".");
            todo!();
        });

        // Initialize

        database.execute_batch(
            &Table::create()
                .if_not_exists()
                .table(NoteTable)
                .col(
                    ColumnDef::new(NoteCharacters::Id)
                        .integer()
                        .primary_key()
                        .not_null()
                        .auto_increment(),
                )
                .col(ColumnDef::new(NoteCharacters::Name).string().not_null())
                .col(ColumnDef::new(NoteCharacters::Tags).json_binary())
                .col(ColumnDef::new(NoteCharacters::Links).json_binary())
                .col(ColumnDef::new(NoteCharacters::Content).text())
                .build(SqliteQueryBuilder),
        )?;

        Ok(Notebook {
            name: name.to_owned(),
            database,
        })
    }
}
