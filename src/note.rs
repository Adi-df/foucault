use std::fs;
use std::path::Path;

use anyhow::Result;
use json::JsonValue;
use thiserror::Error;

use uuid::Uuid;

use rusqlite::Connection;
use sea_query::{Expr, Iden, Query, SqliteQueryBuilder};

use crate::notebook::NoteTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum NoteCharacters {
    Id,
    Name,
    Tags,
    Links,
    Content,
}

#[derive(Clone, Debug, Error)]
pub enum FormatError {
    #[error("The tags were specified in an unknown format : '{tags:?}'")]
    UnknownFormatForTags { tags: String },
    #[error("The links were specified in an unknown format : '{links:?}'")]
    UnknownFormatForLinks { links: String },
}

pub struct Note {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<Uuid>,
    pub content: String,
}

pub struct NoteSummary {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<Uuid>,
}

impl Note {
    pub fn new(name: String, tags: Vec<String>, links: Vec<Uuid>, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            tags,
            links,
            content,
        }
    }

    pub fn insert(&self, db: &Connection) -> Result<()> {
        db.execute_batch(
            &Query::insert()
                .into_table(NoteTable)
                .columns([
                    NoteCharacters::Id,
                    NoteCharacters::Name,
                    NoteCharacters::Tags,
                    NoteCharacters::Links,
                    NoteCharacters::Content,
                ])
                .values([
                    self.id.into(),
                    self.name.as_str().into(),
                    json::stringify(&self.tags[..]).into(),
                    json::stringify(
                        self.links
                            .iter()
                            .map(Uuid::to_string)
                            .collect::<Vec<String>>(),
                    )
                    .into(),
                    self.content.as_str().into(),
                ])?
                .to_string(SqliteQueryBuilder),
        )?;

        Ok(())
    }

    pub fn load(id: Uuid, db: &Connection) -> Result<Self> {
        let [name, raw_tags, raw_links, content]: [String; 4] = db.query_row(
            &Query::select()
                .from(NoteTable)
                .columns([
                    NoteCharacters::Name,
                    NoteCharacters::Tags,
                    NoteCharacters::Links,
                    NoteCharacters::Content,
                ])
                .and_where(Expr::col(NoteCharacters::Id).eq(id))
                .to_string(SqliteQueryBuilder),
            [],
            |row| Ok([row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?]),
        )?;

        let tags = decode_tags(raw_tags.as_str())?;
        let links = decode_links(raw_links.as_str())?;

        Ok(Note {
            id,
            name,
            tags,
            links,
            content,
        })
    }

    pub fn update(&self, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::update()
                .table(NoteTable)
                .values([
                    (NoteCharacters::Name, self.name.as_str().into()),
                    (
                        NoteCharacters::Tags,
                        json::stringify(JsonValue::Array(
                            self.tags
                                .iter()
                                .map(|tag| JsonValue::String(tag.to_string()))
                                .collect::<Vec<JsonValue>>(),
                        ))
                        .into(),
                    ),
                    (
                        NoteCharacters::Links,
                        json::stringify(JsonValue::Array(
                            self.links
                                .iter()
                                .map(|link| JsonValue::String(link.to_string()))
                                .collect::<Vec<JsonValue>>(),
                        ))
                        .into(),
                    ),
                    (NoteCharacters::Content, self.content.as_str().into()),
                ])
                .and_where(Expr::col(NoteCharacters::Id).eq(self.id.to_string()))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?;
        Ok(())
    }

    pub fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.content.as_bytes())?;
        Ok(())
    }

    pub fn import_content(&mut self, file: &Path) -> Result<()> {
        self.content = String::from_utf8(fs::read(file)?)?;
        Ok(())
    }
}

pub fn decode_links(raw_links: &str) -> Result<Vec<Uuid>> {
    let links = json::parse(raw_links)?;
    if links.is_array() {
        links
            .members()
            .map(|link| {
                link.as_str()
                    .ok_or(())
                    .and_then(|str| Uuid::parse_str(str).map_err(|_| ()))
                    .map_err(|()| {
                        FormatError::UnknownFormatForLinks {
                            links: raw_links.to_owned(),
                        }
                        .into()
                    })
            })
            .collect::<Result<Vec<Uuid>>>()
    } else {
        Err(FormatError::UnknownFormatForLinks {
            links: raw_links.to_owned(),
        }
        .into())
    }
}

pub fn decode_tags(raw_tags: &str) -> Result<Vec<String>> {
    let mut tags = json::parse(raw_tags)?;
    if tags.is_array() {
        tags.members_mut()
            .map(JsonValue::take_string)
            .collect::<Option<Vec<String>>>()
            .ok_or(
                FormatError::UnknownFormatForTags {
                    tags: raw_tags.to_owned(),
                }
                .into(),
            )
    } else {
        Err(FormatError::UnknownFormatForTags {
            tags: raw_tags.to_owned(),
        }
        .into())
    }
}
