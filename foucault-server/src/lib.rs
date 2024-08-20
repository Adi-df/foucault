#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

pub mod link_repr;
mod note_api;
pub mod note_repr;
pub mod notebook;
pub mod tag_repr;

use std::sync::Arc;

use anyhow::Result;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
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
