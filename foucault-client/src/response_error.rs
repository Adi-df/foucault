use thiserror::Error;

use reqwest::{Response, StatusCode};

#[derive(Debug, Error)]
pub enum ResponseCodeError {
    #[error("Such operation isn't allowed")]
    Unauthorized,
    #[error("The server encountered an internal error")]
    InternalServerError,
    #[error("Unexpected response code : {0}")]
    UnexpectedCode(StatusCode),
}

pub trait TryResponseCode {
    fn try_response_code(self) -> Result<Self, ResponseCodeError>
    where
        Self: Sized;
}

impl TryResponseCode for Response {
    fn try_response_code(self) -> Result<Self, ResponseCodeError>
    where
        Self: Sized,
    {
        match self.status() {
            StatusCode::OK | StatusCode::NOT_ACCEPTABLE => Ok(self),
            StatusCode::UNAUTHORIZED => Err(ResponseCodeError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ResponseCodeError::InternalServerError),
            status => Err(ResponseCodeError::UnexpectedCode(status)),
        }
    }
}
