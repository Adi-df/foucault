use anyhow::Result;

use rusqlite::Connection;
use sea_query::{ConditionalStatement, Expr, Iden, Query, SqliteQueryBuilder};

#[derive(Iden)]
pub struct TagsTable;

#[derive(Iden)]
pub struct TagsJoinTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum TagsCharacters {
    Id,
    Name,
}

#[derive(Iden, Clone, Copy, Debug)]
pub enum TagsJoinCharacters {
    Id,
    NoteId,
    TagId,
}

#[derive(Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
impl Tag {
    pub fn new(name: &str, db: &Connection) -> Result<Self> {
        db.execute_batch(
            Query::insert()
                .into_table(TagsTable)
                .columns([TagsCharacters::Name])
                .values([name.into()])?
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)?;

        Ok(Self {
            id: db.last_insert_rowid(),
            name: name.to_owned(),
        })
    }

    pub fn tag_exists(name: &str, db: &Connection) -> Result<bool> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .column(TagsCharacters::Id)
                .and_where(Expr::col(TagsCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .exists([])
        .map_err(anyhow::Error::from)
    }

    pub fn delete(self, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::delete()
                .from_table(TagsTable)
                .and_where(Expr::col(TagsCharacters::Id).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?;
        Ok(())
    }
}
