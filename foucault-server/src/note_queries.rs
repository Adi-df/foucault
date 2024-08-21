use std::sync::Arc;

use anyhow::{Error, Result};
use log::info;

use futures::future::join_all;
use sqlx::SqlitePool;

use foucault_core::{
    link_repr::Link,
    note_repr::{Note, NoteError, NoteSummary},
    tag_repr::{Tag, TagError},
};

use crate::tag_queries;

pub(crate) async fn create(name: &str, content: &str, connection: &SqlitePool) -> Result<i64> {
    info!("Insert note {} in the notebook", name);

    if let Some(err) = validate_name(name, connection).await? {
        return Err(err.into());
    };

    let id = sqlx::query!(
        "INSERT INTO notes_table (name, content) VALUES ($1, $2) RETURNING id",
        name,
        content
    )
    .fetch_one(connection)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn validate_name(
    name: &str,
    connection: &SqlitePool,
) -> Result<Option<NoteError>> {
    if name.is_empty() {
        Ok(Some(NoteError::EmptyName))
    } else if name_exists(name, connection).await? {
        Ok(Some(NoteError::AlreadyExists))
    } else {
        Ok(None)
    }
}

pub(crate) async fn name_exists(name: &str, connection: &SqlitePool) -> Result<bool> {
    Ok(
        sqlx::query!("SELECT id FROM notes_table WHERE name=$1", name)
            .fetch_optional(connection)
            .await?
            .is_some(),
    )
}

pub(crate) async fn load_by_id(id: i64, connection: &SqlitePool) -> Result<Option<Note>> {
    sqlx::query!("SELECT name,content FROM notes_table WHERE id=$1", id)
        .fetch_optional(connection)
        .await?
        .map(|row| {
            Ok(Note {
                id,
                name: Arc::from(row.name.expect("There should be a note name")),
                content: Arc::from(row.content.expect("There should be a note content")),
            })
        })
        .transpose()
}

pub(crate) async fn load_by_name(name: &str, connection: &SqlitePool) -> Result<Option<Note>> {
    sqlx::query!("SELECT id, content FROM notes_table WHERE name=$1", name)
        .fetch_optional(connection)
        .await?
        .map(|row| {
            Ok(Note {
                id: row.id.expect("There should be a note id"),
                name: Arc::from(name),
                content: Arc::from(row.content.expect("There should be a note content")),
            })
        })
        .transpose()
}

pub(crate) async fn list_links(id: i64, connection: &SqlitePool) -> Result<Vec<Link>> {
    sqlx::query!("SELECT to_name FROM links_table WHERE from_id=$1", id)
        .fetch_all(connection)
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

pub async fn list_tags(id: i64, connection: &SqlitePool) -> Result<Vec<Tag>> {
    sqlx::query!(
        "SELECT tags_table.id,tags_table.name,tags_table.color FROM tags_join_table INNER JOIN tags_table ON tags_join_table.tag_id = tags_table.id WHERE tags_join_table.note_id=$1",
        id
    )
    .fetch_all(connection)
    .await?
    .into_iter()
    .map(|row| Ok(Tag {
        id: row.id,
        name: row.name,
        color: u32::try_from(row.color)?,
    }))
    .collect()
}

pub(crate) async fn has_tag(id: i64, tag_id: i64, connection: &SqlitePool) -> Result<bool> {
    Ok(sqlx::query!(
        "SELECT tag_id FROM tags_join_table WHERE tag_id=$1 AND note_id=$2",
        tag_id,
        id
    )
    .fetch_optional(connection)
    .await?
    .is_some())
}

pub(crate) async fn rename(id: i64, name: &str, connection: &SqlitePool) -> Result<()> {
    validate_name(name, connection).await?;

    sqlx::query!("UPDATE notes_table SET name=$1 WHERE id=$2", name, id)
        .execute(connection)
        .await?;

    Ok(())
}

pub(crate) async fn delete(id: i64, connection: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM notes_table WHERE id=$1", id)
        .execute(connection)
        .await?;

    Ok(())
}

pub(crate) async fn update_content(id: i64, content: &str, connection: &SqlitePool) -> Result<()> {
    sqlx::query!("UPDATE notes_table SET content=$1 WHERE id=$2", content, id)
        .execute(connection)
        .await?;
    Ok(())
}

pub(crate) async fn update_links(
    id: i64,
    new_links: &[Link],
    connection: &SqlitePool,
) -> Result<()> {
    let current_links = list_links(id, connection).await?;

    join_all(
        current_links
            .iter()
            .filter(|link| !new_links.contains(link))
            .map(|link| {
                sqlx::query!(
                    "DELETE FROM links_table WHERE from_id=$1 AND to_name=$2",
                    id,
                    link.to
                )
                .execute(connection)
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
                    id,
                    link.to
                )
                .execute(connection)
            })
            .collect::<Vec<_>>(),
    )
    .await;

    Ok(())
}

pub(crate) async fn validate_new_tag(
    id: i64,
    tag_id: i64,
    notebook: &SqlitePool,
) -> Result<Option<Error>> {
    if !tag_queries::id_exists(tag_id, notebook).await? {
        Ok(Some(TagError::DoesNotExists.into()))
    } else if has_tag(id, tag_id, notebook).await? {
        Ok(Some(NoteError::NoteAlreadyTagged.into()))
    } else {
        Ok(None)
    }
}

pub(crate) async fn add_tag(id: i64, tag_id: i64, connection: &SqlitePool) -> Result<()> {
    if let Some(err) = validate_new_tag(id, tag_id, connection).await? {
        return Err(err);
    };

    sqlx::query!(
        "INSERT INTO tags_join_table (note_id, tag_id) VALUES ($1, $2)",
        id,
        tag_id
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub(crate) async fn remove_tag(id: i64, tag_id: i64, connection: &SqlitePool) -> Result<()> {
    sqlx::query!(
        "DELETE FROM tags_join_table WHERE note_id=$1 AND tag_id=$2",
        id,
        tag_id
    )
    .execute(connection)
    .await?;

    Ok(())
}

pub(crate) async fn search_by_name(
    pattern: &str,
    connection: &SqlitePool,
) -> Result<Vec<NoteSummary>> {
    let sql_pattern = format!("%{pattern}%");
    join_all(
        sqlx::query!(
            "SELECT id, name FROM notes_table WHERE name LIKE $1 ORDER BY name ASC",
            sql_pattern
        )
        .fetch_all(connection)
        .await?
        .into_iter()
        .map(|row| async move {
            let id = row.id.expect("There should be a note id");
            Ok(NoteSummary {
                id,
                name: Arc::from(row.name.expect("There should be a note name")),
                tags: list_tags(id, connection).await?,
            })
        }),
    )
    .await
    .into_iter()
    .collect()
}

pub(crate) async fn search_by_tag(
    tag_id: i64,
    connection: &SqlitePool,
) -> Result<Vec<NoteSummary>> {
    join_all(
            sqlx::query!(
                "SELECT notes_table.id, notes_table.name FROM tags_join_table INNER JOIN notes_table ON tags_join_table.note_id = notes_table.id WHERE tag_id=$1",
                tag_id
            )
            .fetch_all(connection)
            .await?.
            into_iter()
            .map(|row| async move {
                Ok(NoteSummary {
                    id: row.id,
                    name: Arc::from(row.name.expect("There should be a note name")),
                    tags: list_tags(row.id, connection).await?
                })
            })
        )
        .await
        .into_iter()
        .collect()
}
