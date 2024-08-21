use axum::{
    extract::{Json, State},
    http::StatusCode,
};

use serde_error::Error;

use foucault_core::{
    api::note::{
        AddTagParam, CreateParam, RemoveTagParam, RenameParam, UpdateContentParam,
        UpdateLinksParam, ValidateNewTagParam,
    },
    note_repr::{Note, NoteError, NoteSummary},
    pretty_error,
    tag_repr::{Tag, TagError},
};

use crate::{error::FailibleJsonResult, note_queries, AppState};

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(CreateParam { name, content }): Json<CreateParam>,
) -> FailibleJsonResult<Result<i64, NoteError>> {
    let res = note_queries::create(&name, &content, state.notebook.db()).await;

    match res {
        Ok(id) => Ok((StatusCode::OK, Json::from(Ok(id)))),
        Err(err) => {
            if let Some(note_err) = err.downcast_ref::<NoteError>() {
                Ok((StatusCode::NOT_ACCEPTABLE, Json::from(Err(*note_err))))
            } else {
                pretty_error!("Error encountered during note creation : {}", err);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub(crate) async fn validate_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> FailibleJsonResult<Option<NoteError>> {
    let res = note_queries::validate_name(&name, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered during name validation : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn load_by_id(
    State(state): State<AppState>,
    Json(id): Json<i64>,
) -> FailibleJsonResult<Option<Note>> {
    let res = note_queries::load_by_id(id, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered during note loading : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn load_by_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> FailibleJsonResult<Option<Note>> {
    let res = note_queries::load_by_name(&name, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered during note loading : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn rename(
    State(state): State<AppState>,
    Json(RenameParam { id, name }): Json<RenameParam>,
) -> FailibleJsonResult<Option<NoteError>> {
    let res = note_queries::rename(id, &name, state.notebook.db()).await;

    match res {
        Ok(()) => Ok((StatusCode::OK, Json::from(None))),
        Err(err) => {
            if let Some(note_err) = err.downcast_ref::<NoteError>() {
                Ok((StatusCode::NOT_ACCEPTABLE, Json::from(Some(*note_err))))
            } else {
                pretty_error!("Error encountered during note renaming : {err}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub(crate) async fn delete(State(state): State<AppState>, Json(id): Json<i64>) -> StatusCode {
    let res = note_queries::delete(id, state.notebook.db()).await;

    match res {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            pretty_error!("Error encountered when deleting note : {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn update_content(
    State(state): State<AppState>,
    Json(UpdateContentParam { id, content }): Json<UpdateContentParam>,
) -> StatusCode {
    let res = note_queries::update_content(id, &content, state.notebook.db()).await;

    match res {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            pretty_error!("Error encountered when updating note content : {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn update_links(
    State(state): State<AppState>,
    Json(UpdateLinksParam { id, links }): Json<UpdateLinksParam>,
) -> StatusCode {
    let res = note_queries::update_links(id, &links, state.notebook.db()).await;

    match res {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            pretty_error!("Error encountered when updating note links : {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn list_tags(
    State(state): State<AppState>,
    Json(id): Json<i64>,
) -> FailibleJsonResult<Vec<Tag>> {
    let res = note_queries::list_tags(id, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered while listing note's tags : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn validate_new_tag(
    State(state): State<AppState>,
    Json(ValidateNewTagParam { id, tag_id }): Json<ValidateNewTagParam>,
) -> FailibleJsonResult<Option<Error>> {
    let res = note_queries::validate_new_tag(id, tag_id, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res.map(|err| Error::new(&*err))))),
        Err(err) => {
            pretty_error!("Error encountered during tag validation : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn add_tag(
    State(state): State<AppState>,
    Json(AddTagParam { id, tag_id }): Json<AddTagParam>,
) -> FailibleJsonResult<Option<Error>> {
    let res = note_queries::add_tag(id, tag_id, state.notebook.db()).await;

    match res {
        Ok(()) => Ok((StatusCode::OK, Json::from(None))),
        Err(err) => {
            if err.is::<NoteError>() || err.is::<TagError>() {
                Ok((
                    StatusCode::NOT_ACCEPTABLE,
                    Json::from(Some(Error::new(&*err))),
                ))
            } else {
                pretty_error!("Error encountered while adding tag : {err}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub(crate) async fn remove_tag(
    State(state): State<AppState>,
    Json(RemoveTagParam { id, tag_id }): Json<RemoveTagParam>,
) -> StatusCode {
    let res = note_queries::remove_tag(id, tag_id, state.notebook.db()).await;

    match res {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            pretty_error!("Error encountered while removing tag : {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn search_by_name(
    State(state): State<AppState>,
    Json(pattern): Json<String>,
) -> FailibleJsonResult<Vec<NoteSummary>> {
    let res = note_queries::search_by_name(&pattern, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered when searching notes : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn search_by_tag(
    State(state): State<AppState>,
    Json(tag_id): Json<i64>,
) -> FailibleJsonResult<Vec<NoteSummary>> {
    let res = note_queries::search_by_tag(tag_id, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            pretty_error!("Error encountered while fetching notes summaries : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
