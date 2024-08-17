use std::path::Path;

use futures::future::join_all;
use sqlx::{Row, SqlitePool};
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
        Ok(sqlx::query("SELECT 1 FROM notes_table WHERE name=$1")
            .bind(name)
            .fetch_optional(db)
            .await?
            .is_some())
    }

    pub async fn load_by_id(id: i64, db: &SqlitePool) -> Result<Option<Self>> {
        sqlx::query("SELECT name,content FROM notes_table WHERE id=$1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .map(|row| {
                Ok(Note {
                    id,
                    name: row.try_get(0)?,
                    content: row.try_get(1)?,
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
        sqlx::query("SELECT id, content FROM notes_table WHERE name=$1")
            .bind(name)
            .fetch_optional(db)
            .await?
            .map(|row| {
                Ok(Note {
                    id: row.try_get(0)?,
                    name: name.to_string(),
                    content: row.try_get(1)?,
                })
            })
            .transpose()
    }

    pub async fn list_note_links(id: i64, db: &SqlitePool) -> Result<Vec<Link>> {
        sqlx::query("SELECT to_name FROM links_table WHERE from_id=$1")
            .bind(id)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|row| {
                Ok(Link {
                    from: id,
                    to: row.try_get(0)?,
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
        Ok(
            sqlx::query("SELECT 1 FROM tags_join_table WHERE tag_id=$1 AND note_id=$2")
                .bind(tag_id)
                .bind(self.id)
                .fetch_optional(db)
                .await?
                .is_some(),
        )
    }

    pub async fn rename(&mut self, name: String, db: &SqlitePool) -> Result<()> {
        Note::validate_new_name(&name, db).await?;

        sqlx::query("UPDATE notes_table SET name=$1 WHERE id=$2")
            .bind(&name)
            .bind(self.id)
            .execute(db)
            .await?;

        self.name = name;
        Ok(())
    }

    pub async fn delete(self, db: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM notes_table WHERE id=$1")
            .bind(self.id)
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

        sqlx::query("UPDATE notes_table SET content=$1 WHERE id=$2")
            .bind(&new_content)
            .bind(self.id)
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
                .map(|link| link.to.as_str())
                .map(|link| {
                    sqlx::query("DELETE FROM links_table WHERE from_id=$1 AND to_name IN $2")
                        .bind(self.id)
                        .bind(link)
                        .execute(db)
                })
                .collect::<Vec<_>>(),
        )
        .await;

        join_all(
            new_links
                .iter()
                .filter(|link| !current_links.contains(link))
                .map(|link| &link.to)
                .map(|link| {
                    sqlx::query("INSERT INTO links_table (from_id, to_name) VALUES ($1, $2)")
                        .bind(self.id)
                        .bind(link)
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

        sqlx::query("INSERT INTO tags_join_table (note_id, tag_id) VALUES ($1, $2)")
            .bind(self.id)
            .bind(tag_id)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn remove_tag(&mut self, tag_id: i64, db: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM tags_join_table WHERE note_id=$1 AND tag_id=$2")
            .bind(self.id)
            .bind(tag_id)
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
        join_all(
            sqlx::query("SELECT id,name FROM notes_table WHERE name LIKE $1 ORDER BY name ASC")
                .bind(format!("%{}%", pattern))
                .fetch_all(db)
                .await?
                .into_iter()
                .map(|row| async move {
                    let id = row.try_get(0)?;
                    Ok(NoteSummary {
                        id,
                        name: row.try_get(1)?,
                        tags: Tag::list_note_tags(id, db).await?,
                    })
                }),
        )
        .await
        .into_iter()
        .collect()
    }

    pub async fn fetch_by_tag(tag_id: i64, db: &SqlitePool) -> Result<Vec<NoteSummary>> {
        join_all(sqlx::query("SELECT notes_table.id, notes_table.name FROM tags_join_table INNER JOIN notes_table ON tags_join_table.note_id = notes_table.id WHERE tag_id=$1").bind(tag_id).fetch_all(db).await?.into_iter().map(|row| async move{
            let id = row.try_get(0)?;
            Ok(NoteSummary {
                    id,
                    name: row.try_get(1)?,
                    tags: Tag::list_note_tags(id, db).await?
                })
        })).await.into_iter().collect()
    }
}
