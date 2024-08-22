use thiserror::Error;

use reqwest::{Response, StatusCode};

#[derive(Debug, Error)]
#[error("The request returned unexpected status code : {response_code}")]
pub struct ResponseCodeError {
    response_code: StatusCode,
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
            StatusCode::OK => Ok(self),
            response_code @ _ => Err(ResponseCodeError { response_code }),
        }
    }
}
