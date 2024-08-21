use std::path::Path;

use tokio::fs;

use anyhow::Result;
use serde_error::Error;

use foucault_core::{
    api,
    note_repr::{self, NoteError},
    tag_repr,
};

use crate::{links::Link, tag::Tag, ApiError, NotebookAPI};

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
        let res = notebook
            .client
            .post(notebook.build_url("/note/create"))
            .json(&api::note::CreateParam {
                name: name.clone(),
                content: content.clone(),
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Result<i64, NoteError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        res.map(|id| Self {
            inner: note_repr::Note { id, name, content },
        })
        .map_err(anyhow::Error::from)
    }

    pub async fn validate_name(name: &str, notebook: &NotebookAPI) -> Result<bool> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/validate/name"))
            .json(name)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<NoteError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.is_none())
    }

    pub async fn load_by_id(id: i64, notebook: &NotebookAPI) -> Result<Option<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/load/id"))
            .json(&id)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<note_repr::Note>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.map(Self::from))
    }

    pub async fn load_from_summary(summary: &NoteSummary, notebook: &NotebookAPI) -> Result<Self> {
        match Note::load_by_id(summary.inner.id, notebook).await? {
            Some(note) => Ok(note),
            None => Err(NoteError::DoesNotExist.into()),
        }
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/load/name"))
            .json(name)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<note_repr::Note>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.map(Self::from))
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

    pub async fn tags(&self, notebook: &NotebookAPI) -> Result<Vec<Tag>> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/tag/list"))
            .json(&self.id())
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Vec<tag_repr::Tag>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.into_iter().map(Tag::from).collect())
    }

    pub async fn rename(&mut self, name: String, notebook: &NotebookAPI) -> Result<()> {
        let res = notebook
            .client
            .patch(notebook.build_url("/note/update/name"))
            .json(&api::note::RenameParam {
                id: self.id(),
                name: name.clone(),
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<NoteError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        if let Some(err) = res {
            panic!("The note name is invalid : {err}");
        }

        self.inner.name = name;
        Ok(())
    }

    pub async fn delete(self, notebook: &NotebookAPI) -> Result<()> {
        notebook
            .client
            .delete(notebook.build_url("/note/delete"))
            .json(&self.id())
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?;
        Ok(())
    }

    pub async fn export_content(&self, file: &Path) -> Result<()> {
        fs::write(file, self.inner.content.as_bytes())
            .await
            .map_err(anyhow::Error::from)
    }

    pub async fn import_content(&mut self, file: &Path, notebook: &NotebookAPI) -> Result<()> {
        let new_content = String::from_utf8(fs::read(file).await?)?;

        notebook
            .client
            .patch(notebook.build_url("/note/update/content"))
            .json(&api::note::UpdateContentParam {
                id: self.id(),
                content: new_content.clone(),
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?;

        self.inner.content = new_content;
        Ok(())
    }

    pub async fn update_links(&self, new_links: &[Link], notebook: &NotebookAPI) -> Result<()> {
        notebook
            .client
            .patch(notebook.build_url("/note/update/links"))
            .json(&api::note::UpdateLinksParam {
                id: self.id(),
                links: new_links
                    .iter()
                    .map(|link| link.get_inner().clone())
                    .collect(),
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?;

        Ok(())
    }

    pub async fn validate_tag(&self, tag_id: i64, notebook: &NotebookAPI) -> Result<bool> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/validate/tag"))
            .json(&api::note::ValidateNewTagParam {
                id: self.id(),
                tag_id,
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<Error>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.is_none())
    }

    pub async fn add_tag(&self, tag_id: i64, notebook: &NotebookAPI) -> Result<()> {
        let res = notebook
            .client
            .post(notebook.build_url("/note/tag/add"))
            .json(&api::note::AddTagParam {
                id: self.id(),
                tag_id,
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Option<Error>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        if let Some(err) = res {
            panic!("Failled to add tag : {err}");
        }

        Ok(())
    }

    pub async fn remove_tag(&mut self, tag_id: i64, notebook: &NotebookAPI) -> Result<()> {
        notebook
            .client
            .delete(notebook.build_url("/note/tag/remove"))
            .json(&api::note::RemoveTagParam {
                id: self.id(),
                tag_id,
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?;

        Ok(())
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
        let res = notebook
            .client
            .get(notebook.build_url("/note/search/name"))
            .json(pattern)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Vec<note_repr::NoteSummary>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.into_iter().map(Self::from).collect())
    }

    pub async fn fetch_by_tag(tag_id: i64, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/note/search/tag"))
            .json(&tag_id)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .json::<Vec<note_repr::NoteSummary>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.into_iter().map(Self::from).collect())
    }
}
