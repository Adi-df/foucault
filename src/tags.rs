use sea_query::Iden;

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
    pub id: i64,
    pub name: String,
}
