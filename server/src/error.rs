use actix_web::{http::header::ContentType, HttpResponse};
use serde_json::json;

pub trait SPTFError {
    fn error_code(&self) -> usize;

    fn to_json_string(&self) -> String {
        serde_json::to_string(json!({
            "errorCode": self.error_code()
        }))
        .unwrap()
    }

    fn to_boxed_self<'a>(self) -> Box<dyn SPTFError + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    fn to_http_response(&self) -> HttpResponse {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(self.to_json_string())
    }
}

pub struct UnexpectedError;

impl SPTFError for UnexpectedError {
    fn error_code(&self) -> usize {
        UNEXPECTED_ERROR_CODE
    }
}

pub enum ValidateError {
    NoUsername,
    UnmatchedPassword,
}

impl SPTFError for ValidateError {
    fn error_code(&self) -> usize {
        use ValidateError::*;
        match self {
            NoUsername => VALIDATE_ERROR_NO_USERNAME_ERROR_CODE,
            UnmatchedPassword => VALIDATE_ERROR_UNMATCHED_PASSWORD_ERROR_CODE,
        }
    }
}

pub enum RedisCacheError {
    UpdateAuthTokenFailed,
    ValidateAuthTokenFailed,
}

impl SPTFError for RedisCacheError {
    fn error_code(&self) -> usize {
        use RedisCacheError::*;
        match self {
            UpdateAuthTokenFailed => REDIS_CACHE_ERROR_UPDATE_AUTH_TOKEN_FAILED_ERROR_CODE,
            ValidateAuthTokenFailed => REDIS_CACHE_ERROR_VALIDATE_AUTH_TOKEN_FAILED_ERROR_CODE,
        }
    }
}

const UNEXPECTED_ERROR_CODE: usize = 0x0;
const VALIDATE_ERROR_NO_USERNAME_ERROR_CODE: usize = 0x1;
const VALIDATE_ERROR_UNMATCHED_PASSWORD_ERROR_CODE: usize = 0x2;
const REDIS_CACHE_ERROR_UPDATE_AUTH_TOKEN_FAILED_ERROR_CODE: usize = 0x3;
const REDIS_CACHE_ERROR_VALIDATE_AUTH_TOKEN_FAILED_ERROR_CODE: usize = 0x4;
