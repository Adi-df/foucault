use core::panic;

use axum::extract::Json;
use axum::extract::State;
use axum::http::StatusCode;

use serde::{Deserialize, Serialize};
use serde_error::Error;

use crate::link_repr::Link;
use crate::note_repr;
use crate::note_repr::{Note, NoteError, NoteSummary};
use crate::tag_repr::{Tag, TagError};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateParam {
    name: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameParam {
    id: i64,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContentParam {
    id: i64,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLinksParam {
    id: i64,
    links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateNewTagParam(AddTagParam);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTagParam {
    id: i64,
    tag_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTagParam(AddTagParam);

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(CreateParam { name, content }): Json<CreateParam>,
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
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> (StatusCode, Json<Option<NoteError>>) {
    let res = note_repr::validate_name(&name, state.notebook.db())
        .await
        .expect("Error encountered during name validation");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn load_by_id(
    State(state): State<AppState>,
    Json(id): Json<i64>,
) -> (StatusCode, Json<Option<Note>>) {
    let res = note_repr::load_by_id(id, state.notebook.db())
        .await
        .expect("Error encountered during note loading");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn load_by_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> (StatusCode, Json<Option<Note>>) {
    let res = note_repr::load_by_name(&name, state.notebook.db())
        .await
        .expect("Error encountered during note loading");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn rename(
    State(state): State<AppState>,
    Json(RenameParam { id, name }): Json<RenameParam>,
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

pub(crate) async fn delete(State(state): State<AppState>, Json(id): Json<i64>) -> StatusCode {
    note_repr::delete(id, state.notebook.db())
        .await
        .expect("Error encountered when deleting note");

    StatusCode::OK
}

pub(crate) async fn update_content(
    State(state): State<AppState>,
    Json(UpdateContentParam { id, content }): Json<UpdateContentParam>,
) -> StatusCode {
    note_repr::update_content(id, &content, state.notebook.db())
        .await
        .expect("Error encountered when updating note content");

    StatusCode::OK
}

pub(crate) async fn update_links(
    State(state): State<AppState>,
    Json(UpdateLinksParam { id, links }): Json<UpdateLinksParam>,
) -> StatusCode {
    note_repr::update_links(id, &links, state.notebook.db())
        .await
        .expect("Error encountered when updating note links");

    StatusCode::OK
}

pub(crate) async fn list_tags(
    State(state): State<AppState>,
    Json(id): Json<i64>,
) -> (StatusCode, Json<Vec<Tag>>) {
    let res = note_repr::list_tags(id, state.notebook.db())
        .await
        .expect("Error encountered while listing note's tags");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn validate_new_tag(
    State(state): State<AppState>,
    Json(ValidateNewTagParam(AddTagParam { id, tag_id })): Json<ValidateNewTagParam>,
) -> (StatusCode, Json<Option<Error>>) {
    let res = note_repr::validate_new_tag(id, tag_id, state.notebook.db())
        .await
        .expect("Error encountered during tag validation");

    (StatusCode::OK, Json::from(res.map(|err| Error::new(&*err))))
}

pub(crate) async fn add_tag(
    State(state): State<AppState>,
    Json(AddTagParam { id, tag_id }): Json<AddTagParam>,
) -> (StatusCode, Json<Option<Error>>) {
    let res = note_repr::add_tag(id, tag_id, state.notebook.db()).await;

    match res {
        Ok(()) => (StatusCode::OK, Json::from(None)),
        Err(err) => {
            if err.is::<NoteError>() || err.is::<TagError>() {
                (
                    StatusCode::NOT_ACCEPTABLE,
                    Json::from(Some(Error::new(&*err))),
                )
            } else {
                panic!("Error encountered while adding tag : {}", err);
            }
        }
    }
}

pub(crate) async fn remove_tag(
    State(state): State<AppState>,
    Json(RemoveTagParam(AddTagParam { id, tag_id })): Json<RemoveTagParam>,
) -> StatusCode {
    note_repr::remove_tag(id, tag_id, state.notebook.db())
        .await
        .expect("Error encountered while removing tag");

    StatusCode::OK
}

pub(crate) async fn search_by_name(
    State(state): State<AppState>,
    Json(pattern): Json<String>,
) -> (StatusCode, Json<Vec<NoteSummary>>) {
    let res = note_repr::search_by_name(&pattern, state.notebook.db())
        .await
        .expect("Error encountered when searching notes");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn search_by_tag(
    State(state): State<AppState>,
    Json(tag_id): Json<i64>,
) -> (StatusCode, Json<Vec<NoteSummary>>) {
    let res = note_repr::search_by_tag(tag_id, state.notebook.db())
        .await
        .expect("Error encountered while fetching notes summaries");

    (StatusCode::OK, Json::from(res))
}
