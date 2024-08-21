use log::error;

use axum::{extract::State, http::StatusCode, Json};

use crate::{
    error::FailibleJsonResult,
    tag_repr::{self, Tag, TagError},
    AppState,
};

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> FailibleJsonResult<Result<Tag, TagError>> {
    let res = tag_repr::create(&name, state.notebook.db()).await;

    match res {
        Ok(tag) => Ok((StatusCode::OK, Json::from(Ok(tag)))),
        Err(err) => {
            if let Some(tag_err) = err.downcast_ref::<TagError>() {
                Ok((StatusCode::NOT_ACCEPTABLE, Json::from(Err(*tag_err))))
            } else {
                error!("Error encountered during tag creation : {err}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub(crate) async fn validate_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> FailibleJsonResult<Option<TagError>> {
    let res = tag_repr::validate_name(&name, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            error!("Error encountered during tag creation : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn load_by_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> FailibleJsonResult<Option<Tag>> {
    let res = tag_repr::load_by_name(&name, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            error!("Error encountered while loading tag by name : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn search_by_name(
    State(state): State<AppState>,
    Json(pattern): Json<String>,
) -> FailibleJsonResult<Vec<Tag>> {
    let res = tag_repr::search_by_name(&pattern, state.notebook.db()).await;

    match res {
        Ok(res) => Ok((StatusCode::OK, Json::from(res))),
        Err(err) => {
            error!("Error encountered when searching for tags : {err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub(crate) async fn delete(State(state): State<AppState>, Json(id): Json<i64>) -> StatusCode {
    let res = tag_repr::delete(id, state.notebook.db()).await;

    match res {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            error!("Error encountered when deleting tag : {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
