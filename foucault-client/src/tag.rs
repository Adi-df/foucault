use anyhow::Result;
use thiserror::Error;

use random_color::RandomColor;

use crate::note::NoteSummary;
use crate::NotebookAPI;

#[derive(Debug)]
pub struct Tag {
    id: i64,
    name: String,
    color: u32,
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

impl Tag {
    pub async fn new(name: &str, notebook: &NotebookAPI) -> Result<Self> {
        Tag::validate_new_tag(name, notebook).await?;

        let color = rand_color();
        let id = sqlx::query!(
            "INSERT INTO tags_table (name, color) VALUES ($1, $2) RETURNING id",
            name,
            color
        )
        .fetch_one(db)
        .await?
        .id;

        Ok(Self {
            id,
            name: name.to_string(),
            color,
        })
    }

    pub async fn validate_new_tag(name: &str, notebook: &NotebookAPI) -> Result<()> {
        if name.is_empty() {
            Err(TagError::EmptyName.into())
        } else if Tag::name_exists(name, notebook).await? {
            Err(TagError::AlreadyExists.into())
        } else {
            Ok(())
        }
    }

    pub async fn id_exists(tag_id: i64, notebook: &NotebookAPI) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT id FROM tags_table WHERE id=$1", tag_id)
                .fetch_optional(db)
                .await?
                .is_some(),
        )
    }

    pub async fn name_exists(name: &str, notebook: &NotebookAPI) -> Result<bool> {
        Ok(
            sqlx::query!("SELECT id FROM tags_table WHERE name=$1", name)
                .fetch_optional(db)
                .await?
                .is_some(),
        )
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Tag>> {
        sqlx::query!("SELECT id,color FROM tags_table WHERE name=$1", name)
            .fetch_optional(db)
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

    pub async fn search_by_name(pattern: &str, notebook: &NotebookAPI) -> Result<Vec<Tag>> {
        let sql_pattern = format!("%{pattern}%");
        sqlx::query!(
            "SELECT id,name,color FROM tags_table WHERE name LIKE $1 ORDER BY id DESC",
            sql_pattern
        )
        .fetch_all(db)
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

    pub async fn list_note_tags(note_id: i64, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        sqlx::query!(
            "SELECT tags_table.id,tags_table.name,tags_table.color FROM tags_join_table INNER JOIN tags_table ON tags_join_table.tag_id = tags_table.id WHERE tags_join_table.note_id=$1",
            note_id
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row| Ok(Tag {
            id: row.id,
            name: row.name,
            color: u32::try_from(row.color)?,
        }))
        .collect()
    }

    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn color(&self) -> u32 {
        self.color
    }

    pub async fn delete(self, notebook: &NotebookAPI) -> Result<()> {
        sqlx::query!("DELETE FROM tags_table WHERE id=$1", self.id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn get_related_notes(&self, notebook: &NotebookAPI) -> Result<Vec<NoteSummary>> {
        NoteSummary::fetch_by_tag(self.id, notebook).await
    }
}
