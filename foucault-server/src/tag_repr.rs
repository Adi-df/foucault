use anyhow::Result;

use random_color::RandomColor;

use sqlx::SqlitePool;

use foucault_core::tag_repr::{Tag, TagError};

fn rand_color() -> u32 {
    let [r, g, b] = RandomColor::new().alpha(1.).to_rgb_array();
    (u32::from(r) << 16) + (u32::from(g) << 4) + u32::from(b)
}

pub(crate) async fn create(name: &str, connection: &SqlitePool) -> Result<Tag> {
    if let Some(err) = validate_name(name, connection).await? {
        return Err(err.into());
    };

    let color = rand_color();
    let id = sqlx::query!(
        "INSERT INTO tags_table (name, color) VALUES ($1, $2) RETURNING id",
        name,
        color
    )
    .fetch_one(connection)
    .await?
    .id;

    Ok(Tag {
        id,
        name: name.to_string(),
        color,
    })
}

pub(crate) async fn validate_name(name: &str, connection: &SqlitePool) -> Result<Option<TagError>> {
    if name.is_empty() {
        Ok(Some(TagError::EmptyName))
    } else if name_exists(name, connection).await? {
        Ok(Some(TagError::AlreadyExists))
    } else {
        Ok(None)
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

pub(crate) async fn delete(id: i64, connection: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM tags_table WHERE id=$1", id)
        .execute(connection)
        .await?;
    Ok(())
}
