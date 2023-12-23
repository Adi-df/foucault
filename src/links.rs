use sea_query::Iden;

#[derive(Iden)]
pub struct LinksTable;

#[derive(Iden, Clone, Copy, Debug)]
pub enum LinksCharacters {
    Id,
    Left,
    Right,
}
