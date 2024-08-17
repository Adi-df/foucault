use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use anyhow::Result;
use log::error;
use thiserror::Error;

pub struct Notebook {
    pub name: String,
    file: PathBuf,
    db_pool: SqlitePool,
}

#[derive(Error, Debug)]
pub enum OpeningError {
    #[error("No notebook named {name:?} was found.")]
    NotebookNotFound { name: String },
}

#[derive(Error, Debug)]
pub enum CreationError {
    #[error("Another notebook named {name:?} was found.")]
    NotebookAlreadyExists { name: String },
}

#[derive(Error, Debug)]
pub enum SuppressionError {
    #[error("No notebook named {name:?} was found.")]
    NoNotebookExists { name: String },
}

impl Notebook {
    pub fn db(&self) -> &SqlitePool {
        &self.db_pool
    }

    pub fn dir(&self) -> Option<&Path> {
        self.file.parent()
    }

    pub async fn open_notebook(name: &str, dir: &Path) -> Result<Self> {
        let notebook_path = {
            let app_dir_notebook_path = dir.join(format!("{name}.book"));
            let current_dir_notebook_path = env::current_dir()?.join(format!("{name}.book"));

            if app_dir_notebook_path.exists() {
                app_dir_notebook_path
            } else if current_dir_notebook_path.exists() {
                current_dir_notebook_path
            } else {
                error!("The notebook \"{name}\" was not found.");
                return Err(OpeningError::NotebookNotFound {
                    name: name.to_owned(),
                }
                .into());
            }
        };

        let database = SqlitePoolOptions::new()
            .connect(&format!(
                "sqlite://{}",
                notebook_path
                    .to_str()
                    .expect("The notebook path must be valid unicode")
            ))
            .await
            .unwrap_or_else(|_| {
                error!("Unable to open the notebook \"{name}\".");
                todo!();
            });

        Ok(Notebook {
            name: name.to_owned(),
            file: notebook_path,
            db_pool: database,
        })
    }

    pub async fn new_notebook(name: &str, dir: &Path) -> Result<Self> {
        let notebook_path = dir.join(format!("{name}.book"));

        if notebook_path.try_exists()? {
            error!("A notebook named \"{name}\" already exists.");
            return Err(CreationError::NotebookAlreadyExists {
                name: name.to_owned(),
            }
            .into());
        }

        let database = SqlitePoolOptions::new()
            .connect(&format!(
                "sqlite://{}",
                notebook_path
                    .to_str()
                    .expect("The notebook path must be valid unicode")
            ))
            .await
            .unwrap_or_else(|_| {
                error!("Unable to open the notebook \"{name}\".");
                todo!();
            });

        // Initialize
        sqlx::migrate!().run(&database).await?;

        Ok(Notebook {
            name: name.to_owned(),
            file: notebook_path,
            db_pool: database,
        })
    }

    pub async fn delete_notebook(name: &str, dir: &Path) -> Result<()> {
        let notebook_path = dir.join(format!("{name}.book"));

        if !notebook_path.exists() {
            error!("No notebook named {name} exists.");
            return Err(SuppressionError::NoNotebookExists {
                name: name.to_owned(),
            }
            .into());
        }

        fs::remove_file(notebook_path).await?;
        Ok(())
    }
}
