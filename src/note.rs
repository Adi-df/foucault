use sea_query::Iden;

#[derive(Iden, Clone, Copy, Debug)]
pub enum NoteCharacters {
    Id,
    Name,
    Tags,
    Links,
    Content,
}

pub struct Note {
    pub name: String,
    pub tags: Vec<String>,
    pub links: Vec<usize>,
    pub content: String,
}
