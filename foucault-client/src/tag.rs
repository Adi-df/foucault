use anyhow::Result;

use foucault_core::{
    api,
    tag_repr::{self, TagError},
};

use crate::{response_error::TryResponseCode, ApiError, NotebookAPI};

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
    pub async fn new(name: String, notebook: &NotebookAPI) -> Result<Self> {
        let res = notebook
            .client
            .post(notebook.build_url("/tag/create"))
            .json(&name)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?
            .json::<Result<tag_repr::Tag, TagError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        match res {
            Ok(tag) => Ok(Self::from(tag)),
            Err(err) => {
                panic!("The tag name was invalid : {err}");
            }
        }
    }

    pub async fn validate_name(name: &str, notebook: &NotebookAPI) -> Result<bool> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/validate/name"))
            .json(name)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?
            .json::<Option<TagError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.is_none())
    }

    pub async fn load_by_name(name: &str, notebook: &NotebookAPI) -> Result<Option<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/load/name"))
            .json(name)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?
            .json::<Option<tag_repr::Tag>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        Ok(res.map(Self::from))
    }

    pub async fn search_by_name(pattern: &str, notebook: &NotebookAPI) -> Result<Vec<Self>> {
        let res = notebook
            .client
            .get(notebook.build_url("/tag/search/name"))
            .json(pattern)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?
            .json::<Vec<tag_repr::Tag>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

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

    pub async fn rename(id: i64, name: String, notebook: &NotebookAPI) -> Result<()> {
        let res = notebook
            .client
            .patch(notebook.build_url("/tag/update/name"))
            .json(&api::tag::RenameParam {
                id,
                name: name.clone(),
            })
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?
            .json::<Option<TagError>>()
            .await
            .map_err(ApiError::UnableToParseResponse)?;

        if let Some(err) = res {
            panic!("The tag name is invalid : {err}");
        }

        Ok(())
    }

    pub async fn delete(id: i64, notebook: &NotebookAPI) -> Result<()> {
        notebook
            .client
            .delete(notebook.build_url("/tag/delete"))
            .json(&id)
            .send()
            .await
            .map_err(ApiError::UnableToContactRemoteNotebook)?
            .try_response_code()?;

        Ok(())
    }
}
