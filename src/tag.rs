use anyhow::Result;
use thiserror::Error;

use random_color::RandomColor;

use crate::note::NoteSummary;

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
    pub fn new(name: &str, db: &Connection) -> Result<Self> {
        Tag::validate_new_tag(name, db)?;

        let color = rand_color();
        db.execute_batch(
            Query::insert()
                .into_table(TagsTable)
                .columns([TagsCharacters::Name, TagsCharacters::Color])
                .values([name.into(), color.into()])?
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )
        .map_err(anyhow::Error::from)?;

        Ok(Self {
            id: db.last_insert_rowid(),
            name: name.to_owned(),
            color,
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
                .columns([TagsCharacters::Id, TagsCharacters::Color])
                .and_where(Expr::col(TagsCharacters::Name).eq(name))
                .to_string(SqliteQueryBuilder)
                .as_str(),
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(anyhow::Error::from)
        .map(|res| {
            res.map(|(id, color)| Tag {
                id,
                name: name.to_string(),
                color,
            })
        })
    }

    pub fn search_by_name(pattern: &str, db: &Connection) -> Result<Vec<Tag>> {
        db.prepare(
            Query::select()
                .from(TagsTable)
                .columns([
                    TagsCharacters::Id,
                    TagsCharacters::Name,
                    TagsCharacters::Color,
                ])
                .order_by(TagsCharacters::Id, Order::Desc)
                .and_where(Expr::col(TagsCharacters::Name).like(format!("%{pattern}%")))
                .to_string(SqliteQueryBuilder)
                .as_str(),
        )?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .map(|row| -> Result<(i64, String, u32)> { row.map_err(anyhow::Error::from) })
        .map(|row| row.map(|(id, name, color)| Tag { id, name, color }))
        .collect()
    }

    pub fn list_note_tags(note_id: i64, db: &Connection) -> Result<Vec<Self>> {
        db.prepare(
            Query::select()
                .from(TagsJoinTable)
                .columns([
                    (TagsTable, TagsCharacters::Id),
                    (TagsTable, TagsCharacters::Name),
                    (TagsTable, TagsCharacters::Color),
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
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .map(|row| {
            row.map(|(id, name, color)| Self { id, name, color })
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
    pub fn color(&self) -> u32 {
        self.color
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
