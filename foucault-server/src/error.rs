use axum::{http::StatusCode, Json};

pub type FailibleJsonResult<T> = Result<(StatusCode, Json<T>), StatusCode>;
