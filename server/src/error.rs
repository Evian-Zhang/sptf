use crate::protos::sptf::ErrorResponse;
use actix_web::{http::header::ContentType, HttpResponse};
use log::error;
use protobuf::Message;
use serde_json::json;

pub trait SPTFError {
    fn error_code(&self) -> usize;

    fn to_json_string(&self) -> String {
        serde_json::to_string(&json!({
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
        HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(self.to_json_string())
    }

    fn to_proto_error(&self) -> ErrorResponse {
        let mut error_response = ErrorResponse::default();
        error_response.set_error_code(self.error_code() as u64);
        error_response
    }

    fn to_proto_binary(&self) -> Vec<u8> {
        let error_response = self.to_proto_error();
        error_response.write_to_bytes().unwrap_or_else(|err| {
            error!("Failed to write error to binary: {}", err);
            vec![]
        })
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
    WrongCookie,
}

impl SPTFError for ValidateError {
    fn error_code(&self) -> usize {
        use ValidateError::*;
        match self {
            NoUsername => VALIDATE_ERROR_NO_USERNAME_ERROR_CODE,
            UnmatchedPassword => VALIDATE_ERROR_UNMATCHED_PASSWORD_ERROR_CODE,
            WrongCookie => VALIDATE_ERROR_WRONG_COOKIE_ERROR_CODE,
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

pub enum FileError {
    PermissionDenied,
}

impl SPTFError for FileError {
    fn error_code(&self) -> usize {
        use FileError::*;
        match self {
            PermissionDenied => FILE_ERROR_PERMISSION_DENIED_ERROR_CODE,
        }
    }
}

pub enum ProtobufError {
    WrongFormat,
}

impl SPTFError for ProtobufError {
    fn error_code(&self) -> usize {
        use ProtobufError::*;
        match self {
            WrongFormat => PROTOBUF_ERROR_WRONG_FORMAT_ERROR_CODE,
        }
    }
}

const UNEXPECTED_ERROR_CODE: usize = 0x0;
const VALIDATE_ERROR_NO_USERNAME_ERROR_CODE: usize = 0x1;
const VALIDATE_ERROR_UNMATCHED_PASSWORD_ERROR_CODE: usize = 0x2;
const VALIDATE_ERROR_WRONG_COOKIE_ERROR_CODE: usize = 0x3;
const REDIS_CACHE_ERROR_UPDATE_AUTH_TOKEN_FAILED_ERROR_CODE: usize = 0x4;
const REDIS_CACHE_ERROR_VALIDATE_AUTH_TOKEN_FAILED_ERROR_CODE: usize = 0x5;
const FILE_ERROR_PERMISSION_DENIED_ERROR_CODE: usize = 0x6;
const PROTOBUF_ERROR_WRONG_FORMAT_ERROR_CODE: usize = 0x7;
