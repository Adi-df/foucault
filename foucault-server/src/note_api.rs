use core::panic;

use anyhow::Error;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::link_repr::Link;
use crate::note_repr;
use crate::note_repr::{Note, NoteError, NoteSummary};
use crate::tag_repr::TagError;
use crate::AppState;

pub struct CreateParam {
    name: String,
    content: String,
}

pub struct RenameParam {
    id: i64,
    name: String,
}

pub struct UpdateContentParam {
    id: i64,
    content: String,
}

pub struct UpdateLinksParam {
    id: i64,
    links: Vec<Link>,
}

pub type ValidateNewTagParam = AddTagParam;

pub struct AddTagParam {
    id: i64,
    tag_id: i64,
}

pub type RemoveTagParam = AddTagParam;

pub(crate) async fn create(
    Json(CreateParam { name, content }): Json<CreateParam>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Result<i64, NoteError>>) {
    let res = note_repr::create(&name, &content, state.notebook.db()).await;

    match res {
        Ok(id) => (StatusCode::OK, Json::from(Ok(id))),
        Err(err) => {
            if let Some(note_err) = err.downcast_ref::<NoteError>() {
                (StatusCode::NOT_ACCEPTABLE, Json::from(Err(*note_err)))
            } else {
                panic!("Error encountered during note creation : {}", err);
            }
        }
    }
}

pub(crate) async fn validate_name(
    Json(name): Json<String>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<NoteError>>) {
    let res = note_repr::validate_name(&name, state.notebook.db())
        .await
        .expect("Error encountered during name validation");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn load_by_id(
    Json(id): Json<i64>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<Note>>) {
    let res = note_repr::load_by_id(id, state.notebook.db())
        .await
        .expect("Error encountered during note loading");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn load_by_name(
    Json(name): Json<String>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<Note>>) {
    let res = note_repr::load_by_name(&name, state.notebook.db())
        .await
        .expect("Error encountered during note loading");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn rename(
    Json(RenameParam { id, name }): Json<RenameParam>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<NoteError>>) {
    let res = note_repr::rename(id, &name, state.notebook.db()).await;

    match res {
        Ok(()) => (StatusCode::OK, Json::from(None)),
        Err(err) => {
            if let Some(note_err) = err.downcast_ref::<NoteError>() {
                (StatusCode::NOT_ACCEPTABLE, Json::from(Some(*note_err)))
            } else {
                panic!("Error encountered during note renaming : {}", err);
            }
        }
    }
}

pub(crate) async fn delete(Json(id): Json<i64>, State(state): State<AppState>) -> StatusCode {
    note_repr::delete(id, state.notebook.db())
        .await
        .expect("Error encountered when deleting note");

    StatusCode::OK
}

pub(crate) async fn update_content(
    Json(UpdateContentParam { id, content }): Json<UpdateContentParam>,
    State(state): State<AppState>,
) -> StatusCode {
    note_repr::update_content(id, &content, state.notebook.db())
        .await
        .expect("Error encountered when updating note content");

    StatusCode::OK
}

pub(crate) async fn update_links(
    Json(UpdateLinksParam { id, links }): Json<UpdateLinksParam>,
    State(state): State<AppState>,
) -> StatusCode {
    note_repr::update_links(id, &links, state.notebook.db())
        .await
        .expect("Error encountered when updating note links");

    StatusCode::OK
}

pub(crate) async fn validate_new_tag(
    Json(ValidateNewTagParam { id, tag_id }): Json<ValidateNewTagParam>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<Error>>) {
    let res = note_repr::validate_new_tag(id, tag_id, state.notebook.db())
        .await
        .expect("Error encountered during tag validation");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn add_tag(
    Json(AddTagParam { id, tag_id }): Json<AddTagParam>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Option<Error>>) {
    let res = note_repr::add_tag(id, tag_id, state.notebook.db()).await;

    match res {
        Ok(()) => (StatusCode::OK, Json::from(None)),
        Err(err) => {
            if err.is::<NoteError>() || err.is::<TagError>() {
                (StatusCode::NOT_ACCEPTABLE, Json::from(Some(err)))
            } else {
                panic!("Error encountered while adding tag : {}", err);
            }
        }
    }
}

pub(crate) async fn remove_tag(
    Json(RemoveTagParam { id, tag_id }): Json<RemoveTagParam>,
    State(state): State<AppState>,
) -> StatusCode {
    note_repr::remove_tag(id, tag_id, state.notebook.db())
        .await
        .expect("Error encountered while removing tag");

    StatusCode::OK
}

pub(crate) async fn search_by_name(
    Json(pattern): Json<String>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<NoteSummary>>) {
    let res = note_repr::search_by_name(&pattern, state.notebook.db())
        .await
        .expect("Error encountered when searching notes");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn fetch_by_tag(
    Json(tag_id): Json<i64>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<NoteSummary>>) {
    let res = note_repr::fetch_by_tag(tag_id, state.notebook.db())
        .await
        .expect("Error encountered while fetching notes summaries");

    (StatusCode::OK, Json::from(res))
}
