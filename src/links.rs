use anyhow::Result;

use rusqlite::Connection;
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Iden, SqliteQueryBuilder, Table};

use crate::helpers::DiscardResult;
use crate::note::{NotesCharacters, NotesTable};

#[derive(Iden)]
pub struct LinksTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum LinksCharacters {
    Id,
    FromId,
    ToName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub from: i64,
    pub to: String,
}

impl LinksTable {
    pub fn create(db: &Connection) -> Result<()> {
        db.execute_batch(
            Table::create()
                .if_not_exists()
                .table(LinksTable)
                .col(
                    ColumnDef::new(LinksCharacters::Id)
                        .integer()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(LinksCharacters::FromId).integer().not_null())
                .col(ColumnDef::new(LinksCharacters::ToName).string().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .from(LinksTable, LinksCharacters::FromId)
                        .to(NotesTable, NotesCharacters::Id)
                        .on_update(ForeignKeyAction::Cascade)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder)
                .as_str(),
        )
        .discard_result()
    }
}
