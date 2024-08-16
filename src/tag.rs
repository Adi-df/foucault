use anyhow::Result;

use rusqlite::{Connection, OptionalExtension};
use sea_query::{
    ColumnDef, Expr, ForeignKey, ForeignKeyAction, Iden, JoinType, Order, Query,
    SqliteQueryBuilder, Table,
};
use thiserror::Error;

use crate::helpers::DiscardResult;
use crate::note::{NoteSummary, NotesCharacters, NotesTable};

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
    id: i64,
    name: String,
}

#[derive(Debug, Error)]
pub enum TagError {
    #[error("A simillarly named tag already exists")]
    AlreadyExists,
    #[error("The provided tag name is empty")]
    EmptyName,
    #[error("No such tag exists")]
    DoesNotExists,
}

impl Tag {
    pub fn new(name: &str, db: &Connection) -> Result<Self> {
        Tag::validate_new_tag(name, db)?;

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

    pub fn validate_new_tag(name: &str, db: &Connection) -> Result<()> {
        if name.is_empty() {
            Err(TagError::EmptyName.into())
        } else if Tag::name_exists(name, db)? {
            Err(TagError::AlreadyExists.into())
        } else {
            Ok(())
        }
    }

    pub fn id_exists(tag_id: i64, db: &Connection) -> Result<bool> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .column(TagsCharacters::Id)
                .and_where(Expr::col(TagsCharacters::Id).eq(tag_id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .exists([])
        .map_err(anyhow::Error::from)
    }

    pub fn name_exists(name: &str, db: &Connection) -> Result<bool> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .column(TagsCharacters::Name)
                .and_where(Expr::col(TagsCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .exists([])
        .map_err(anyhow::Error::from)
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

    pub fn search_by_name(pattern: &str, db: &Connection) -> Result<Vec<Tag>> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .columns([TagsCharacters::Id, TagsCharacters::Name])
                .order_by(TagsCharacters::Id, Order::Desc)
                .and_where(Expr::col(TagsCharacters::Name).like(format!("%{pattern}%")))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .map(|row| -> Result<(i64, String)> { row.map_err(anyhow::Error::from) })
        .map(|row| row.map(|(id, name)| Tag { id, name }))
        .collect()
    }

    pub fn list_note_tags(note_id: i64, db: &Connection) -> Result<Vec<Self>> {
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
                .and_where(Expr::col(TagsJoinCharacters::NoteId).eq(note_id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .map(|row| {
            row.map(|(id, name)| Self { id, name })
                .map_err(anyhow::Error::from)
        })
        .collect()
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
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

    pub fn get_related_notes(&self, db: &Connection) -> Result<Vec<NoteSummary>> {
        NoteSummary::fetch_by_tag(self.id, db)
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
