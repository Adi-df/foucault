use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use log::error;
use sea_query::ForeignKey;
use sea_query::ForeignKeyAction;
use sea_query::Order;
use thiserror::Error;

use rusqlite::Connection;
use sea_query::{ColumnDef, Expr, Query, SqliteQueryBuilder, Table};

use crate::links::LinksCharacters;
use crate::links::LinksTable;
use crate::note::Note;
use crate::note::NoteTable;
use crate::note::{NoteCharacters, NoteSummary};
use crate::tags::TagsCharacters;
use crate::tags::TagsJoinCharacters;
use crate::tags::TagsJoinTable;
use crate::tags::TagsTable;

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

        database.execute_batch(
            Table::create()
                .if_not_exists()
                .table(NoteTable)
                .col(
                    ColumnDef::new(NoteCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(NoteCharacters::Name).string().not_null())
                .col(ColumnDef::new(NoteCharacters::Content).text())
                .build(SqliteQueryBuilder)
                .as_str(),
        )?;

        database.execute_batch(
            Table::create()
                .if_not_exists()
                .table(TagsTable)
                .col(
                    ColumnDef::new(TagsCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(TagsCharacters::Name).string().not_null())
                .build(SqliteQueryBuilder)
                .as_str(),
        )?;

        database.execute_batch(
            Table::create()
                .if_not_exists()
                .table(TagsJoinTable)
                .col(
                    ColumnDef::new(TagsJoinCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(
                    ColumnDef::new(TagsJoinCharacters::NoteId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(TagsJoinCharacters::TagId)
                        .integer()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(TagsJoinTable, TagsJoinCharacters::NoteId)
                        .to(NoteTable, NoteCharacters::Id)
                        .on_update(ForeignKeyAction::Cascade)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(TagsJoinTable, TagsJoinCharacters::TagId)
                        .to(TagsTable, TagsCharacters::Id)
                        .on_update(ForeignKeyAction::Cascade)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder)
                .as_str(),
        )?;

        database.execute_batch(
            Table::create()
                .if_not_exists()
                .table(LinksTable)
                .col(
                    ColumnDef::new(LinksCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(LinksCharacters::Left).integer().not_null())
                .col(ColumnDef::new(LinksCharacters::Right).integer().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from(LinksTable, LinksCharacters::Left)
                        .to(NoteTable, NoteCharacters::Id)
                        .on_update(ForeignKeyAction::Cascade)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(LinksTable, LinksCharacters::Right)
                        .to(NoteTable, NoteCharacters::Id)
                        .on_update(ForeignKeyAction::Cascade)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder)
                .as_str(),
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
                    .columns([NoteCharacters::Id, NoteCharacters::Name])
                    .order_by(NoteCharacters::Name, Order::Asc)
                    .and_where(Expr::col(NoteCharacters::Name).like(format!("{pattern}%")))
                    .to_string(SqliteQueryBuilder)
                    .as_str(),
            )?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .map(|row| -> Result<(i64, String)> { row.map_err(anyhow::Error::from) })
            .map(|row| {
                row.and_then(|(id, name)| {
                    Ok(NoteSummary {
                        id,
                        name,
                        tags: Note::fetch_tags(id, self.db())?,
                    })
                })
            })
            .collect::<Result<Vec<NoteSummary>>>()
    }
}
