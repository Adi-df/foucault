use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use rusqlite::{Connection, OptionalExtension};
use sea_query::{ColumnDef, Expr, Iden, JoinType, Order, Query, SqliteQueryBuilder, Table};

use crate::helpers::DiscardResult;
use crate::links::{Link, LinksCharacters, LinksTable};
use crate::tag::{Tag, TagError, TagsJoinCharacters, TagsJoinTable};

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
    id: i64,
    name: String,
    content: String,
}

#[derive(Debug)]
pub struct NoteSummary {
    id: i64,
    name: String,
    tags: Vec<Tag>,
}

#[derive(Debug, Error)]
pub enum NoteError {
    #[error("No such note exists")]
    DoesNotExist,
    #[error("A similarly named note already exists")]
    AlreadyExists,
    #[error("The provided note name is empty")]
    EmptyName,
    #[error("The note already has the provided tag")]
    NoteAlreadyTagged,
}

impl Note {
    pub fn new(name: String, content: String, db: &Connection) -> Result<Self> {
        Note::validate_new_name(&name, db)?;

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

    pub fn validate_new_name(name: &str, db: &Connection) -> Result<()> {
        if name.is_empty() {
            return Err(NoteError::EmptyName.into());
        }

        if Note::name_exists(name, db)? {
            return Err(NoteError::AlreadyExists.into());
        }

        Ok(())
    }

    pub fn name_exists(name: &str, db: &Connection) -> Result<bool> {
        db.prepare(
            Query::select()
                .from(NotesTable)
                .column(NotesCharacters::Id)
                .and_where(Expr::col(NotesCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .exists([])
        .map_err(anyhow::Error::from)
    }

    pub fn load_by_id(id: i64, db: &Connection) -> Result<Option<Self>> {
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

    pub fn load_from_summary(summary: &NoteSummary, db: &Connection) -> Result<Self> {
        match Note::load_by_id(summary.id, db)? {
            Some(note) => Ok(note),
            None => Err(NoteError::DoesNotExist.into()),
        }
    }

    pub fn load_by_name(name: &str, db: &Connection) -> Result<Option<Self>> {
        db.query_row(
            Query::select()
                .from(NotesTable)
                .columns([NotesCharacters::Id, NotesCharacters::Content])
                .and_where(Expr::col(NotesCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(anyhow::Error::from)
        .map(|res| {
            res.map(|(id, content)| Note {
                id,
                name: name.to_string(),
                content,
            })
        })
    }

    pub fn list_note_links(id: i64, db: &Connection) -> Result<Vec<Link>> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([LinksCharacters::ToName])
                .and_where(Expr::col(LinksCharacters::FromId).eq(id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| row.get(0))?
        .map(|row| {
            row.map_err(anyhow::Error::from)
                .map(|to| Link { from: id, to })
        })
        .collect()
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn links(&self, db: &Connection) -> Result<Vec<Link>> {
        Note::list_note_links(self.id, db)
    }

    pub fn tags(&self, db: &Connection) -> Result<Vec<Tag>> {
        Tag::list_note_tags(self.id, db)
    }

    pub fn has_tag(&self, tag_id: i64, db: &Connection) -> Result<bool> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([TagsJoinCharacters::TagId, TagsJoinCharacters::NoteId])
                .and_where(Expr::col(TagsJoinCharacters::TagId).eq(tag_id))
                .and_where(Expr::col(TagsJoinCharacters::NoteId).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .exists([])
        .map_err(anyhow::Error::from)
    }

    pub fn rename(&mut self, name: String, db: &Connection) -> Result<()> {
        Note::validate_new_name(&name, db)?;

        db.execute_batch(
            Query::update()
                .table(NotesTable)
                .values([(NotesCharacters::Name, self.name.as_str().into())])
                .and_where(Expr::col(NotesCharacters::Id).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)?;
        self.name = name;
        Ok(())
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

    pub fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.content.as_bytes()).map_err(anyhow::Error::from)
    }

    pub fn import_content(&mut self, file: &Path, db: &Connection) -> Result<()> {
        let new_content = String::from_utf8(fs::read(file)?)?;

        db.execute_batch(
            Query::update()
                .table(NotesTable)
                .values([(NotesCharacters::Content, self.content.as_str().into())])
                .and_where(Expr::col(NotesCharacters::Id).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)?;

        self.content = new_content;
        Ok(())
    }

    pub fn update_links(&self, new_links: &[Link], db: &Connection) -> Result<()> {
        let current_links = self.links(db)?;

        let removed = current_links
            .iter()
            .filter(|link| !new_links.contains(link))
            .map(|link| &link.to)
            .peekable();

        db.execute_batch(
            Query::delete()
                .from_table(LinksTable)
                .and_where(Expr::col(LinksCharacters::FromId).eq(self.id))
                .and_where(Expr::col(LinksCharacters::ToName).is_in(removed))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?;

        let mut added = new_links
            .iter()
            .filter(|link| !current_links.contains(link))
            .map(|link| &link.to)
            .peekable();

        if added.peek().is_some() {
            db.execute_batch({
                let mut builder = Query::insert()
                    .into_table(LinksTable)
                    .columns([LinksCharacters::FromId, LinksCharacters::ToName])
                    .to_owned();
                for new_link in added {
                    builder.values([self.id.into(), new_link.into()])?;
                }
                builder.to_string(SqliteQueryBuilder).as_str()
            })?;
        }

        Ok(())
    }

    pub fn validate_new_tag(&self, tag_id: i64, db: &Connection) -> Result<()> {
        if !Tag::id_exists(tag_id, db)? {
            Err(TagError::DoesNotExists.into())
        } else if self.has_tag(tag_id, db)? {
            Err(NoteError::NoteAlreadyTagged.into())
        } else {
            Ok(())
        }
    }

    pub fn add_tag(&self, tag_id: i64, db: &Connection) -> Result<()> {
        self.validate_new_tag(tag_id, db)?;
        db.execute_batch(
            Query::insert()
                .into_table(TagsJoinTable)
                .columns([TagsJoinCharacters::NoteId, TagsJoinCharacters::TagId])
                .values([self.id.into(), tag_id.into()])?
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }

    pub fn remove_tag(&mut self, tag_id: i64, db: &Connection) -> Result<()> {
        db.execute_batch(
            Query::delete()
                .from_table(TagsJoinTable)
                .and_where(Expr::col(TagsJoinCharacters::TagId).eq(tag_id))
                .and_where(Expr::col(TagsJoinCharacters::NoteId).eq(self.id))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)
    }
}

impl NoteSummary {
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    pub fn search_by_name(pattern: &str, db: &Connection) -> Result<Vec<Self>> {
        db.prepare(
            Query::select()
                .from(NotesTable)
                .columns([NotesCharacters::Id, NotesCharacters::Name])
                .order_by(NotesCharacters::Name, Order::Asc)
                .and_where(Expr::col(NotesCharacters::Name).like(format!("%{pattern}%")))
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
                    tags: Tag::list_note_tags(id, db)?,
                })
            })
        })
        .collect()
    }

    pub fn fetch_by_tag(tag_id: i64, db: &Connection) -> Result<Vec<NoteSummary>> {
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
                .and_where(Expr::col(TagsJoinCharacters::TagId).eq(tag_id))
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
                    tags: Tag::list_note_tags(id, db)?,
                })
            })
        })
        .collect()
    }
}

impl NotesTable {
    pub fn create(db: &Connection) -> Result<()> {
        db.execute_batch(
            Table::create()
                .if_not_exists()
                .table(NotesTable)
                .col(
                    ColumnDef::new(NotesCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(
                    ColumnDef::new(NotesCharacters::Name)
                        .string()
                        .unique_key()
                        .not_null(),
                )
                .col(ColumnDef::new(NotesCharacters::Content).text())
                .build(SqliteQueryBuilder)
                .as_str(),
        )
        .discard_result()
    }
}
