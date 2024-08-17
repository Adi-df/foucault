use std::path::Path;

use futures::future::join_all;
use sqlx::SqlitePool;
use tokio::fs;

use anyhow::Result;
use thiserror::Error;

use crate::links::Link;
use crate::tag::{Tag, TagError};

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
    pub async fn new(name: String, content: String, db: &SqlitePool) -> Result<Self> {
        Note::validate_new_name(&name, db).await?;

        let ref_name = &name;
        let ref_content = &content;
        let id = sqlx::query!(
            "INSERT INTO notes_table (name, content) VALUES ($1, $2) RETURNING id",
            ref_name,
            ref_content
        )
        .fetch_one(db)
        .await?
        .id;

        Ok(Self { id, name, content })
    }

    pub async fn validate_new_name(name: &str, db: &SqlitePool) -> Result<()> {
        if name.is_empty() {
            return Err(NoteError::EmptyName.into());
        }

        if Note::name_exists(name, db).await? {
            return Err(NoteError::AlreadyExists.into());
        }

        Ok(())
    }

    pub async fn name_exists(name: &str, db: &SqlitePool) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT id FROM notes_table WHERE name=$1", name)
                .fetch_optional(db)
                .await?
                .is_some(),
        )
    }

    pub async fn load_by_id(id: i64, db: &SqlitePool) -> Result<Option<Self>> {
        sqlx::query!("SELECT name,content FROM notes_table WHERE id=$1", id)
            .fetch_optional(db)
            .await?
            .map(|row| {
                Ok(Note {
                    id,
                    name: row.name.expect("There should be a note name"),
                    content: row.content.expect("There should be a note content"),
                })
            })
            .transpose()
    }

    pub async fn load_from_summary(summary: &NoteSummary, db: &SqlitePool) -> Result<Self> {
        match Note::load_by_id(summary.id, db).await? {
            Some(note) => Ok(note),
            None => Err(NoteError::DoesNotExist.into()),
        }
    }

    pub async fn load_by_name(name: &str, db: &SqlitePool) -> Result<Option<Self>> {
        sqlx::query!("SELECT id,content FROM notes_table WHERE name=$1", name)
            .fetch_optional(db)
            .await?
            .map(|row| {
                Ok(Note {
                    id: row.id.expect("There should be a note id"),
                    name: name.to_string(),
                    content: row.content.expect("There should be a note content"),
                })
            })
            .transpose()
    }

    pub async fn list_note_links(id: i64, db: &SqlitePool) -> Result<Vec<Link>> {
        sqlx::query!("SELECT to_name FROM links_table WHERE from_id=$1", id)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| {
                Ok(Link {
                    from: id,
                    to: row.to_name,
                })
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

    pub async fn links(&self, db: &SqlitePool) -> Result<Vec<Link>> {
        Note::list_note_links(self.id, db).await
    }

    pub async fn tags(&self, db: &SqlitePool) -> Result<Vec<Tag>> {
        Tag::list_note_tags(self.id, db).await
    }

    pub async fn has_tag(&self, tag_id: i64, db: &SqlitePool) -> Result<bool> {
        Ok(sqlx::query!(
            "SELECT tag_id FROM tags_join_table WHERE tag_id=$1 AND note_id=$2",
            tag_id,
            self.id
        )
        .fetch_optional(db)
        .await?
        .is_some())
    }

    pub async fn rename(&mut self, name: String, db: &SqlitePool) -> Result<()> {
        Note::validate_new_name(&name, db).await?;

        let ref_name = &name;
        sqlx::query!(
            "UPDATE notes_table SET name=$1 WHERE id=$2",
            ref_name,
            self.id
        )
        .execute(db)
        .await?;

        self.name = name;
        Ok(())
    }

    pub async fn delete(self, db: &SqlitePool) -> Result<()> {
        sqlx::query!("DELETE FROM notes_table WHERE id=$1", self.id)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.content.as_bytes())
            .await
            .map_err(anyhow::Error::from)
    }

    pub async fn import_content(&mut self, file: &Path, db: &SqlitePool) -> Result<()> {
        let new_content = String::from_utf8(fs::read(file).await?)?;

        let ref_new_content = &new_content;
        sqlx::query!(
            "UPDATE notes_table SET content=$1 WHERE id=$2",
            ref_new_content,
            self.id
        )
        .execute(db)
        .await?;

        self.content = new_content;
        Ok(())
    }

    pub async fn update_links(&self, new_links: &[Link], db: &SqlitePool) -> Result<()> {
        let current_links = self.links(db).await?;

        join_all(
            current_links
                .iter()
                .filter(|link| !new_links.contains(link))
                .map(|link| {
                    sqlx::query!(
                        "DELETE FROM links_table WHERE from_id=$1 AND to_name=$2",
                        self.id,
                        link.to
                    )
                    .execute(db)
                })
                .collect::<Vec<_>>(),
        )
        .await;

        join_all(
            new_links
                .iter()
                .filter(|link| !current_links.contains(link))
                .map(|link| {
                    sqlx::query!(
                        "INSERT INTO links_table (from_id, to_name) VALUES ($1, $2)",
                        self.id,
                        link.to
                    )
                    .execute(db)
                })
                .collect::<Vec<_>>(),
        )
        .await;

        Ok(())
    }

    pub async fn validate_new_tag(&self, tag_id: i64, db: &SqlitePool) -> Result<()> {
        if !Tag::id_exists(tag_id, db).await? {
            Err(TagError::DoesNotExists.into())
        } else if self.has_tag(tag_id, db).await? {
            Err(NoteError::NoteAlreadyTagged.into())
        } else {
            Ok(())
        }
    }

    pub async fn add_tag(&self, tag_id: i64, db: &SqlitePool) -> Result<()> {
        self.validate_new_tag(tag_id, db).await?;

        sqlx::query!(
            "INSERT INTO tags_join_table (note_id, tag_id) VALUES ($1, $2)",
            self.id,
            tag_id
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn remove_tag(&mut self, tag_id: i64, db: &SqlitePool) -> Result<()> {
        sqlx::query!(
            "DELETE FROM tags_join_table WHERE note_id=$1 AND tag_id=$2",
            self.id,
            tag_id
        )
        .execute(db)
        .await?;

        Ok(())
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

    pub async fn search_by_name(pattern: &str, db: &SqlitePool) -> Result<Vec<Self>> {
        let sql_pattern = format!("%{}%", pattern);
        join_all(
            sqlx::query!(
                "SELECT id,name FROM notes_table WHERE name LIKE $1 ORDER BY name ASC",
                sql_pattern
            )
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| async move {
                let id = row.id.expect("There should be a note id");
                Ok(NoteSummary {
                    id,
                    name: row.name.expect("There should be a note name"),
                    tags: Tag::list_note_tags(id, db).await?,
                })
            }),
        )
        .await
        .into_iter()
        .collect()
    }

    pub async fn fetch_by_tag(tag_id: i64, db: &SqlitePool) -> Result<Vec<NoteSummary>> {
        join_all(sqlx::query!("SELECT notes_table.id, notes_table.name FROM tags_join_table INNER JOIN notes_table ON tags_join_table.note_id = notes_table.id WHERE tag_id=$1", tag_id).fetch_all(db).await?.into_iter().map(|row| async move{
            Ok(NoteSummary {
                    id: row.id,
                    name: row.name.expect("There should be a note name"),
                    tags: Tag::list_note_tags(row.id, db).await?
                })
        })).await.into_iter().collect()
    }
}
