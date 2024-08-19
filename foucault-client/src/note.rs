use std::path::Path;

use tokio::fs;

use anyhow::Result;

use foucault_server::note_repr;
use foucault_server::note_repr::NoteError;

use crate::links::Link;
use crate::tag::Tag;
use crate::NotebookAPI;

#[derive(Debug)]
pub struct Note {
    inner: note_repr::Note,
}

impl From<note_repr::Note> for Note {
    fn from(inner: note_repr::Note) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Clone)]
pub struct NoteSummary {
    inner: note_repr::NoteSummary,
}

impl From<note_repr::NoteSummary> for NoteSummary {
    fn from(inner: note_repr::NoteSummary) -> Self {
        Self { inner }
    }
}

impl Note {
    pub async fn new(name: String, content: String, notebook: &NotebookAPI) -> Result<Self> {
        todo!();
    }

    pub async fn validate_new_name(name: &str, notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }

    pub async fn load_by_id(id: i64, notebook: &NotebookAPI) -> Result<Option<Self>> {
        todo!();
    }

    pub async fn load_from_summary(summary: &NoteSummary, notebook: &NotebookAPI) -> Result<Self> {
        match Note::load_by_id(summary.inner.id, notebook).await? {
            Some(note) => Ok(note),
            None => Err(NoteError::DoesNotExist.into()),
        }
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Self>> {
        todo!();
    }

    pub async fn list_note_links(id: i64, notebook: &NotebookAPI) -> Result<Vec<Link>> {
        todo!();
    }

    pub fn id(&self) -> i64 {
        self.inner.id
    }
    pub fn name(&self) -> &str {
        &self.inner.name
    }
    pub fn content(&self) -> &str {
        &self.inner.content
    }

    pub async fn links(&self, notebook: &NotebookAPI) -> Result<Vec<Link>> {
        Note::list_note_links(self.inner.id, notebook).await
    }

    pub async fn tags(&self, notebook: &NotebookAPI) -> Result<Vec<Tag>> {
        Tag::list_note_tags(self.inner.id, notebook).await
    }

    pub async fn has_tag(&self, tag_id: i64, notebook: &NotebookAPI) -> Result<bool> {
        todo!();
    }

    pub async fn rename(&mut self, name: String, notebook: &NotebookAPI) -> Result<()> {
        todo!();
        self.inner.name = name;
    }

    pub async fn delete(self, notebook: &NotebookAPI) -> Result<()> {
        todo!()
    }

    pub async fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.inner.content.as_bytes())
            .await
            .map_err(anyhow::Error::from)
    }

    pub async fn import_content(&mut self, file: &Path, notebook: &NotebookAPI) -> Result<()> {
        let new_content = String::from_utf8(fs::read(file).await?)?;

        todo!();

        self.inner.content = new_content;
        Ok(())
    }

    pub async fn update_links(&self, new_links: &[Link], notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }

    pub async fn validate_new_tag(&self, tag_id: i64, notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }

    pub async fn add_tag(&self, tag_id: i64, notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }

    pub async fn remove_tag(&mut self, tag_id: i64, notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }
}

impl NoteSummary {
    pub fn id(&self) -> i64 {
        self.inner.id
    }
    pub fn name(&self) -> &str {
        &self.inner.name
    }
    pub fn tags(&self) -> Vec<Tag> {
        self.inner.tags.iter().cloned().map(Tag::from).collect()
    }

    pub async fn search_by_name(pattern: &str, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        todo!();
    }

    pub async fn fetch_by_tag(tag_id: i64, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        todo!();
    }
}
