use anyhow::Result;

use uuid::Uuid;

use rusqlite::Connection;
use sea_query::{Iden, Query, SqliteQueryBuilder};

use crate::notebook::NoteTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum NoteCharacters {
    Id,
    Name,
    Tags,
    Links,
    Content,
}

pub struct Note {
    pub id: Uuid,
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<usize>,
    pub content: String,
}

impl Note {
    pub fn new(name: String, tags: Vec<String>, links: Vec<usize>, content: String) -> Self {
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
                .values_panic([
                    self.id.into(),
                    self.name.as_str().into(),
                    json::stringify(&self.tags[..]).into(),
                    json::stringify(&self.links[..]).into(),
                    self.content.as_str().into(),
                ])
                .to_string(SqliteQueryBuilder),
        )?;

        Ok(())
    }
}
