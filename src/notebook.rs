use std::path::Path;

use log::error;

use polodb_core::Database;

pub struct Notebook {
    name: String,
    database: Database,
}

#[derive(Debug)]
pub enum OpeningError {
    NotebookNotFound { name: String },
}

impl Notebook {
    pub fn open_notebook(name: &str, dir: &Path) -> Result<Self, OpeningError> {
        let notebook_path = dir.join(name);

        if !notebook_path.exists() {
            error!("The notebook \"{name}\" was not found.");
            return Err(OpeningError::NotebookNotFound {
                name: name.to_owned(),
            });
        }

        let database = Database::open_file(notebook_path).unwrap_or_else(|_| {
            error!("Unable to open the notebook \"{name}\".");
            todo!();
        });

        Ok(Notebook {
            name: name.to_owned(),
            database,
        })
    }
}
