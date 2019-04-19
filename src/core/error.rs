use std::{fmt, io};

use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    BadRequest(String),

    #[display(fmt = "r2d2 pool error: {}", _0)]
    R2D2Error(r2d2::Error),

    #[display(fmt = "connection error: {}", _0)]
    ConnectionError(diesel::ConnectionError),

    #[display(fmt = "diesel error: {}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "toml error: {}", _0)]
    TomlError(toml::de::Error),

    #[display(fmt = "io error: {}", _0)]
    IoError(io::Error),

    #[display(fmt = "block execute timeout")]
    Timeout,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        use Error::*;

        match *self {
            BadRequest(ref message) => {
                let payload = json!({
                    "title": "请求内容错误".to_string(),
                    "message": message.to_string()
                });
                HttpResponse::BadRequest().json(payload)
            }
            ref err => {
                let payload = json!({ "message": format!("{}", err) });
                HttpResponse::InternalServerError().json(payload)
            }
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::BadRequest(err)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(err: &'a str) -> Error {
        Error::from(err.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(err: diesel::ConnectionError) -> Self {
        Error::ConnectionError(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DieselError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlError(err)
    }
}

#[allow(non_snake_case)]
pub fn BadRequest<E>(err: E) -> Error
where
    E: 'static + fmt::Display,
{
    Error::BadRequest(format!("{}", err))
}
