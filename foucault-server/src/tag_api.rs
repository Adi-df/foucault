use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::tag_repr;
use crate::tag_repr::{Tag, TagError};
use crate::AppState;

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> (StatusCode, Json<Result<Tag, TagError>>) {
    let res = tag_repr::create(&name, state.notebook.db()).await;

    match res {
        Ok(tag) => (StatusCode::OK, Json::from(Ok(tag))),
        Err(err) => {
            if let Some(tag_err) = err.downcast_ref::<TagError>() {
                (StatusCode::NOT_ACCEPTABLE, Json::from(Err(*tag_err)))
            } else {
                panic!("Error encountered during tag creation : {}", err);
            }
        }
    }
}

pub(crate) async fn validate_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> (StatusCode, Json<Option<TagError>>) {
    let res = tag_repr::validate_name(&name, state.notebook.db())
        .await
        .expect("Error encountered when validating tag name");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn load_by_name(
    State(state): State<AppState>,
    Json(name): Json<String>,
) -> (StatusCode, Json<Option<Tag>>) {
    let res = tag_repr::load_by_name(&name, state.notebook.db())
        .await
        .expect("Error encountered while loading tag by name");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn search_by_name(
    State(state): State<AppState>,
    Json(pattern): Json<String>,
) -> (StatusCode, Json<Vec<Tag>>) {
    let res = tag_repr::search_by_name(&pattern, state.notebook.db())
        .await
        .expect("Error encountered when searching for tags");

    (StatusCode::OK, Json::from(res))
}

pub(crate) async fn delete(State(state): State<AppState>, Json(id): Json<i64>) -> StatusCode {
    tag_repr::delete(id, state.notebook.db())
        .await
        .expect("Error encountered when deleting tag");
    StatusCode::OK
}
