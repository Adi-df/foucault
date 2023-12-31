use sea_query::Iden;

#[derive(Iden)]
pub struct LinksTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum LinksCharacters {
    Id,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub from: i64,
    pub to: i64,
}
