#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod note_api;
mod note_queries;
pub mod notebook;
mod tag_api;
mod tag_queries;

use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use tokio::{io, net::TcpListener};

use foucault_core::{permissions::Permissions, NotebookApiInfo};

use crate::notebook::Notebook;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Unable to bind the listener : {0}")]
    UnableToBindListener(io::Error),
    #[error("An internal server error occured : {0}")]
    InternalServerError(io::Error),
}

#[derive(Clone)]
struct AppState {
    notebook: Arc<Notebook>,
    permissions: Permissions,
}

pub async fn serve(notebook: Arc<Notebook>, permissions: Permissions, port: u16) -> Result<()> {
    let state = AppState {
        notebook,
        permissions,
    };
    let app = Router::new()
        .route("/notebook", get(notebook_info))
        .route("/note/create", post(note_api::create))
        .route("/note/delete", delete(note_api::delete))
        .route("/note/validate/name", get(note_api::validate_name))
        .route("/note/validate/tag", get(note_api::validate_new_tag))
        .route("/note/load/id", get(note_api::load_by_id))
        .route("/note/load/name", get(note_api::load_by_name))
        .route("/note/search/name", get(note_api::search_by_name))
        .route("/note/search/tag", get(note_api::search_by_tag))
        .route("/note/update/name", patch(note_api::rename))
        .route("/note/update/content", patch(note_api::update_content))
        .route("/note/update/links", patch(note_api::update_links))
        .route("/note/tag/list", get(note_api::list_tags))
        .route("/note/tag/add", post(note_api::add_tag))
        .route("/note/tag/remove", delete(note_api::remove_tag))
        .route("/tag/create", post(tag_api::create))
        .route("/tag/delete", delete(tag_api::delete))
        .route("/tag/validate/name", get(tag_api::validate_name))
        .route("/tag/load/name", get(tag_api::load_by_name))
        .route("/tag/search/name", get(tag_api::search_by_name))
        .with_state(state);

    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&address)
        .await
        .map_err(ServerError::UnableToBindListener)?;
    axum::serve(listener, app)
        .await
        .map_err(ServerError::InternalServerError)?;

    Ok(())
}

async fn notebook_info(State(state): State<AppState>) -> (StatusCode, Json<NotebookApiInfo>) {
    (
        StatusCode::OK,
        Json::from(NotebookApiInfo {
            name: state.notebook.name.clone(),
            permissions: state.permissions,
        }),
    )
}
