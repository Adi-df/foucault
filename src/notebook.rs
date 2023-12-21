use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use log::error;
use sea_query::Order;
use thiserror::Error;
use uuid::Uuid;

use rusqlite::Connection;
use sea_query::{ColumnDef, Expr, Iden, Query, SqliteQueryBuilder, Table};

use crate::note::decode_links;
use crate::note::decode_tags;
use crate::note::{NoteCharacters, NoteSummary};

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

#[derive(Iden)]
pub struct NoteTable;

impl Notebook {
    pub fn db(&self) -> &Connection {
        &self.database
    }

    pub fn file(&self) -> &Path {
        &self.file
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

        database.execute_batch(
            &Table::create()
                .if_not_exists()
                .table(NoteTable)
                .col(ColumnDef::new(NoteCharacters::Id).uuid().primary_key())
                .col(ColumnDef::new(NoteCharacters::Name).string().not_null())
                .col(ColumnDef::new(NoteCharacters::Tags).json_binary())
                .col(ColumnDef::new(NoteCharacters::Links).json_binary())
                .col(ColumnDef::new(NoteCharacters::Content).text())
                .build(SqliteQueryBuilder),
        )?;

        Ok(Notebook {
            name: name.to_owned(),
            file: notebook_path,
            database,
        })
    }

    pub fn delete_notebook(name: &str, dir: &Path) -> Result<()> {
        let notebook_path = dir.join(name);

        if !notebook_path.exists() {
            error!("No notebook named \"{name}\" exists.");
            return Err(SuppressionError::NoNotebookExists {
                name: name.to_owned(),
            }
            .into());
        }

        fs::remove_file(notebook_path)?;
        Ok(())
    }

    pub fn search_name(&self, pattern: &str) -> Result<Vec<NoteSummary>> {
        self.database
            .prepare(
                Query::select()
                    .from(NoteTable)
                    .columns([
                        NoteCharacters::Id,
                        NoteCharacters::Name,
                        NoteCharacters::Tags,
                        NoteCharacters::Links,
                    ])
                    .order_by(NoteCharacters::Name, Order::Asc)
                    .and_where(Expr::col(NoteCharacters::Name).like(format!("{pattern}%")))
                    .to_string(SqliteQueryBuilder)
                    .as_str(),
            )?
            .query_map([], |row| {
                Ok([row.get(0)?, row.get(1)?, row.get(2)?, row.get(2)?])
            })?
            .map(|row| -> Result<[String; 4]> { row.map_err(anyhow::Error::from) })
            .map(|row| {
                row.and_then(|[raw_id, name, raw_tags, raw_links]| {
                    let id = Uuid::from_str(raw_id.as_str()).map_err(anyhow::Error::from)?;
                    let tags = decode_tags(raw_tags.as_str())?;
                    let links = decode_links(raw_links.as_str())?;

                    Ok(NoteSummary {
                        id,
                        name,
                        tags,
                        links,
                    })
                })
            })
            .collect::<Result<Vec<NoteSummary>>>()
    }
}
