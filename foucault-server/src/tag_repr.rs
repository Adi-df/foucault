use anyhow::Result;
use thiserror::Error;

use random_color::RandomColor;

use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: u32,
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

fn rand_color() -> u32 {
    let [r, g, b] = RandomColor::new().alpha(1.).to_rgb_array();
    (u32::from(r) << 16) + (u32::from(g) << 4) + u32::from(b)
}

pub(crate) async fn create(name: &str, connection: &SqlitePool) -> Result<i64> {
    validate_name(name, connection).await?;

    let color = rand_color();
    let id = sqlx::query!(
        "INSERT INTO tags_table (name, color) VALUES ($1, $2) RETURNING id",
        name,
        color
    )
    .fetch_one(connection)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn validate_name(name: &str, connection: &SqlitePool) -> Result<()> {
    if name.is_empty() {
        Err(TagError::EmptyName.into())
    } else if name_exists(name, connection).await? {
        Err(TagError::AlreadyExists.into())
    } else {
        Ok(())
    }
}

pub(crate) async fn id_exists(id: i64, connection: &SqlitePool) -> Result<bool> {
    Ok(sqlx::query!("SELECT id FROM tags_table WHERE id=$1", id)
        .fetch_optional(connection)
        .await?
        .is_some())
}

pub(crate) async fn name_exists(name: &str, connection: &SqlitePool) -> Result<bool> {
    Ok(
        sqlx::query!("SELECT id FROM tags_table WHERE name=$1", name)
            .fetch_optional(connection)
            .await?
            .is_some(),
    )
}

pub(crate) async fn load_by_name(name: &str, connection: &SqlitePool) -> Result<Option<Tag>> {
    sqlx::query!("SELECT id,color FROM tags_table WHERE name=$1", name)
        .fetch_optional(connection)
        .await?
        .map(|row| {
            Ok(Tag {
                id: row.id.expect("There should be a tag id"),
                name: name.to_string(),
                color: u32::try_from(row.color)?,
            })
        })
        .transpose()
}

pub(crate) async fn search_by_name(pattern: &str, connection: &SqlitePool) -> Result<Vec<Tag>> {
    let sql_pattern = format!("%{pattern}%");
    sqlx::query!(
        "SELECT id,name,color FROM tags_table WHERE name LIKE $1 ORDER BY id DESC",
        sql_pattern
    )
    .fetch_all(connection)
    .await?
    .into_iter()
    .map(|row| {
        Ok(Tag {
            id: row.id,
            name: row.name,
            color: u32::try_from(row.color)?,
        })
    })
    .collect()
}

pub async fn list_note_tags(note_id: i64, connection: &SqlitePool) -> Result<Vec<Tag>> {
    sqlx::query!(
        "SELECT tags_table.id,tags_table.name,tags_table.color FROM tags_join_table INNER JOIN tags_table ON tags_join_table.tag_id = tags_table.id WHERE tags_join_table.note_id=$1",
        note_id
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

pub(crate) async fn delete(id: i64, connection: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM tags_table WHERE id=$1", id)
        .execute(connection)
        .await?;
    Ok(())
}
