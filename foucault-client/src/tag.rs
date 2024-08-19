use anyhow::Result;

use crate::note::NoteSummary;
use crate::NotebookAPI;

#[derive(Debug)]
pub struct Tag {
    id: i64,
    name: String,
    color: u32,
}

impl Tag {
    pub async fn new(name: &str, notebook: &NotebookAPI) -> Result<Self> {
        todo!();
    }

    pub async fn validate_new_tag(name: &str, notebook: &NotebookAPI) -> Result<()> {
        todo!();
    }

    pub async fn id_exists(id: i64, notebook: &NotebookAPI) -> Result<bool> {
        todo!();
    }

    pub async fn name_exists(name: &str, notebook: &NotebookAPI) -> Result<bool> {
        todo!();
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Self>> {
        todo!();
    }

    pub async fn search_by_name(pattern: &str, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        todo!();
    }

    pub async fn list_note_tags(note_id: i64, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        todo!();
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
        todo!();
    }

    pub async fn get_related_notes(&self, notebook: &NotebookAPI) -> Result<Vec<NoteSummary>> {
        NoteSummary::fetch_by_tag(self.id, notebook).await
    }
}
