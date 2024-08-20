#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod link_repr;
pub mod note_api;
pub mod note_repr;
pub mod notebook;
pub mod tag_repr;

use std::sync::Arc;

use anyhow::Result;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tokio::net::TcpListener;

use crate::notebook::Notebook;

#[derive(Clone)]
struct AppState {
    notebook: Arc<Notebook>,
}

pub async fn serve(notebook: Arc<Notebook>) -> Result<()> {
    let state = AppState { notebook };
    let app = Router::new()
        .route("/name", get(notebook_name))
        .route("/note/create", post(note_api::create))
        .route("/note/delete", delete(note_api::delete))
        .route("/note/validate/name", get(note_api::validate_name))
        .route("/note/validate/tag", get(note_api::validate_new_tag))
        .route("/note/load/id", get(note_api::load_by_id))
        .route("/note/load/name", get(note_api::load_by_name))
        .route("/note/search/name", get(note_api::search_by_name))
        .route("/note/search/tag", get(note_api::search_by_tag))
        .route("/note/update/name", put(note_api::rename))
        .route("/note/update/content", put(note_api::update_content))
        .route("/note/update/links", put(note_api::update_links))
        .route("/note/tag/add", post(note_api::add_tag))
        .route("/note/tag/remove", delete(note_api::remove_tag))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8078")
        .await
        .expect("Listener should bind successfuly");
    axum::serve(listener, app)
        .await
        .map_err(anyhow::Error::from)
}

async fn notebook_name(State(state): State<AppState>) -> (StatusCode, String) {
    (StatusCode::OK, state.notebook.name.clone())
}
