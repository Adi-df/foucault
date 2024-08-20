use anyhow::Result;

use foucault_server::tag_repr::{self, TagError};

use crate::{note::NoteSummary, NotebookAPI};

#[derive(Debug, Clone)]
pub struct Tag {
    inner: tag_repr::Tag,
}

impl From<tag_repr::Tag> for Tag {
    fn from(inner: tag_repr::Tag) -> Self {
        Self { inner }
    }
}

impl Tag {
    pub async fn new(name: &str, notebook: &NotebookAPI) -> Result<Self> {
        let res = notebook
            .client
            .post(notebook.build_url("/tag/create"))
            .json(&name)
            .send()
            .await?
            .json::<Result<tag_repr::Tag, TagError>>()
            .await?;

        match res {
            Ok(tag) => Ok(Self::from(tag)),
            Err(err) => {
                panic!("The tag name was invalid : {}", err);
            }
        }
    }

    pub async fn validate_new_tag(name: &str, notebook: &NotebookAPI) -> Result<bool> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/validate/name"))
            .json(name)
            .send()
            .await?
            .json::<Option<TagError>>()
            .await?;

        Ok(res.is_none())
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/load/name"))
            .json(name)
            .send()
            .await?
            .json::<Option<tag_repr::Tag>>()
            .await?;

        Ok(res.map(Self::from))
    }

    pub async fn search_by_name(pattern: &str, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/search/name"))
            .json(pattern)
            .send()
            .await?
            .json::<Vec<tag_repr::Tag>>()
            .await?;

        Ok(res.into_iter().map(Self::from).collect())
    }

    pub fn id(&self) -> i64 {
        self.inner.id
    }
    pub fn name(&self) -> &str {
        &self.inner.name
    }
    pub fn color(&self) -> u32 {
        self.inner.color
    }

    pub async fn delete(self, notebook: &NotebookAPI) -> Result<()> {
        notebook
            .client
            .delete(notebook.build_url("/tag/delete"))
            .json(&self.id())
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_related_notes(&self, notebook: &NotebookAPI) -> Result<Vec<NoteSummary>> {
        NoteSummary::fetch_by_tag(self.id(), notebook).await
    }
}
