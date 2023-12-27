use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use rusqlite::{Connection, OptionalExtension};
use sea_query::{Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder};

use crate::helpers::TryFromDatabase;
use crate::links::LinksCharacters;
use crate::tag::{Tag, TagsCharacters, TagsJoinCharacters, TagsJoinTable, TagsTable};

#[derive(Iden)]
pub struct NotesTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum NotesCharacters {
    Id,
    Name,
    Content,
}

#[derive(Debug)]
pub struct Note {
    pub id: i64,
    pub name: String,
    pub content: String,
}

#[derive(Debug)]
pub struct NoteSummary {
    pub id: i64,
    pub name: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct NoteData {
    pub note: Note,
    pub tags: Vec<Tag>,
    pub links: Vec<i64>,
}

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("No such note exists")]
    NoteDoesNotExist,
}

impl Note {
    pub fn new(name: String, content: String, db: &Connection) -> Result<Self> {
        db.execute_batch(
            Query::insert()
                .into_table(NotesTable)
                .columns([NotesCharacters::Name, NotesCharacters::Content])
                .values([name.as_str().into(), content.as_str().into()])?
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?;

        Ok(Self {
            id: db.last_insert_rowid(),
            name,
            content,
        })
    }

    pub fn load(id: i64, db: &Connection) -> Result<Option<Self>> {
        db.query_row(
            Query::select()
                .from(NotesTable)
                .columns([NotesCharacters::Name, NotesCharacters::Content])
                .and_where(Expr::col(NotesCharacters::Id).eq(id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
            [],
            |row| Ok([row.get(0)?, row.get(1)?]),
        )
        .optional()
        .map_err(anyhow::Error::from)
        .map(|res| res.map(|[name, content]| Note { id, name, content }))
    }

    pub fn update(&self, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::update()
                .table(NotesTable)
                .values([
                    (NotesCharacters::Name, self.name.as_str().into()),
                    (NotesCharacters::Content, self.content.as_str().into()),
                ])
                .and_where(Expr::col(NotesCharacters::Id).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }

    pub fn delete(self, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::delete()
                .from_table(NotesTable)
                .and_where(Expr::col(NotesCharacters::Id).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }

    pub fn add_tag(&self, tag: &Tag, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::insert()
                .into_table(TagsJoinTable)
                .columns([TagsJoinCharacters::NoteId, TagsJoinCharacters::TagId])
                .values([self.id.into(), tag.id.into()])?
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }

    pub fn fetch_tags(id: i64, db: &Connection) -> Result<Vec<Tag>> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([
                    (TagsTable, TagsCharacters::Id),
                    (TagsTable, TagsCharacters::Name),
                ])
                .join(
                    JoinType::InnerJoin,
                    TagsTable,
                    Expr::col((TagsTable, TagsCharacters::Id))
                        .equals((TagsJoinTable, TagsJoinCharacters::TagId)),
                )
                .and_where(Expr::col(TagsJoinCharacters::NoteId).eq(id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .map(|row| {
            row.map(|(id, name)| Tag { id, name })
                .map_err(anyhow::Error::from)
        })
        .collect()
    }

    pub fn fetch_links(id: i64, db: &Connection) -> Result<Vec<i64>> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([LinksCharacters::Right])
                .and_where(Expr::col(LinksCharacters::Left).eq(id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| row.get(0))?
        .map(|row| row.map_err(anyhow::Error::from))
        .collect()
    }

    pub fn get_tags(&self, db: &Connection) -> Result<Vec<Tag>> {
        Self::fetch_tags(self.id, db)
    }

    pub fn get_links(&self, db: &Connection) -> Result<Vec<i64>> {
        Self::fetch_links(self.id, db)
    }

    pub fn remove_tag(&self, id: i64, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::delete()
                .from_table(TagsJoinTable)
                .and_where(
                    Expr::col(TagsJoinCharacters::TagId)
                        .eq(id)
                        .and(Expr::col(TagsJoinCharacters::NoteId).eq(self.id)),
                )
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }

    pub fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.content.as_bytes()).map_err(anyhow::Error::from)
    }

    pub fn import_content(&mut self, file: &Path) -> Result<()> {
        self.content = String::from_utf8(fs::read(file)?)?;
        Ok(())
    }

    pub fn search_by_name(pattern: &str, db: &Connection) -> Result<Vec<NoteSummary>> {
        db.prepare(
            Query::select()
                .from(NotesTable)
                .columns([NotesCharacters::Id, NotesCharacters::Name])
                .order_by(NotesCharacters::Name, Order::Asc)
                .and_where(Expr::col(NotesCharacters::Name).like(format!("{pattern}%")))
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
                    tags: Note::fetch_tags(id, db)?,
                })
            })
        })
        .collect()
    }
}

impl TryFromDatabase<NoteSummary> for Note {
    fn try_from_database(note_summary: NoteSummary, db: &Connection) -> Result<Self> {
        if let Some(note) = Note::load(note_summary.id, db)? {
            Ok(note)
        } else {
            Err(NoteError::NoteDoesNotExist.into())
        }
    }
}

impl From<NoteData> for Note {
    fn from(note_data: NoteData) -> Self {
        note_data.note
    }
}

impl TryFromDatabase<Note> for NoteSummary {
    fn try_from_database(note: Note, db: &Connection) -> Result<Self> {
        Ok(NoteSummary {
            id: note.id,
            tags: note.get_tags(db)?,
            name: note.name,
        })
    }
}

impl TryFromDatabase<Note> for NoteData {
    fn try_from_database(note: Note, db: &Connection) -> Result<Self> {
        Ok(NoteData {
            tags: note.get_tags(db)?,
            links: note.get_links(db)?,
            note,
        })
    }
}
