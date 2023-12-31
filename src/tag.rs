use anyhow::Result;

use rusqlite::{Connection, OptionalExtension};
use sea_query::{
    ColumnDef, Expr, ForeignKey, ForeignKeyAction, Iden, JoinType, Order, Query,
    SqliteQueryBuilder, Table,
};

use crate::helpers::DiscardResult;
use crate::note::{Note, NoteSummary, NotesCharacters, NotesTable};

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

    pub fn load_by_name(name: &str, db: &Connection) -> Result<Option<Tag>> {
        db.query_row(
            Query::select()
                .from(TagsTable)
                .columns([TagsCharacters::Id])
                .and_where(Expr::col(TagsCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(anyhow::Error::from)
        .map(|res| {
            res.map(|id| Tag {
                id,
                name: name.to_string(),
            })
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

    pub fn search_by_name(pattern: &str, db: &Connection) -> Result<Vec<Tag>> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .columns([TagsCharacters::Id, TagsCharacters::Name])
                .order_by(TagsCharacters::Id, Order::Desc)
                .and_where(Expr::col(TagsCharacters::Name).like(format!("{pattern}%")))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .map(|row| -> Result<(i64, String)> { row.map_err(anyhow::Error::from) })
        .map(|row| row.map(|(id, name)| Tag { id, name }))
        .collect()
    }

    pub fn fetch_notes(id: i64, db: &Connection) -> Result<Vec<NoteSummary>> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([
                    (NotesTable, NotesCharacters::Id),
                    (NotesTable, NotesCharacters::Name),
                ])
                .join(
                    JoinType::InnerJoin,
                    NotesTable,
                    Expr::col((TagsJoinTable, TagsJoinCharacters::NoteId))
                        .equals((NotesTable, NotesCharacters::Id)),
                )
                .and_where(Expr::col(TagsJoinCharacters::TagId).eq(id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .map(|row| row.map_err(anyhow::Error::from))
        .map(|row| {
            row.and_then(|(id, name)| {
                Ok(NoteSummary {
                    id,
                    name,
                    tags: Note::list_tags(id, db)?,
                })
            })
        })
        .collect()
    }

    pub fn get_notes(&self, db: &Connection) -> Result<Vec<NoteSummary>> {
        Tag::fetch_notes(self.id, db)
    }
}

impl TagsTable {
    pub fn create(db: &Connection) -> Result<()> {
        db.execute_batch(
            Table::create()
                .if_not_exists()
                .table(TagsTable)
                .col(
                    ColumnDef::new(TagsCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(
                    ColumnDef::new(TagsCharacters::Name)
                        .string()
                        .unique_key()
                        .not_null(),
                )
                .build(SqliteQueryBuilder)
                .as_str(),
        )
        .discard_result()
    }
}

impl TagsJoinTable {
    pub fn create(db: &Connection) -> Result<()> {
        db.execute_batch(
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
                        .to(NotesTable, NotesCharacters::Id)
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
        )
        .discard_result()
    }
}
