use anyhow::Result;

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
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<usize>,
    pub content: String,
}

impl Note {
    pub fn insert(&self, db: &Connection) -> Result<()> {
        db.execute_batch(
            &Query::insert()
                .into_table(NoteTable)
                .columns([
                    NoteCharacters::Name,
                    NoteCharacters::Tags,
                    NoteCharacters::Links,
                    NoteCharacters::Content,
                ])
                .values_panic([
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
