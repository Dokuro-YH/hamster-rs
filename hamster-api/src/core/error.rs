use std::fmt;

use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    BadRequest(String),

    #[display(fmt = "r2d2 pool error: {}", _0)]
    R2D2Error(r2d2::Error),

    #[display(fmt = "diesel error: {}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "execute timeout")]
    Timeout,

    #[display(fmt = "服务器内部错误")]
    InternalServerError(String),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        use Error::*;

        match *self {
            BadRequest(_) => HttpResponse::BadRequest().finish(),
            _ => HttpResponse::InternalServerError().finish(),
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

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DieselError(err)
    }
}

#[allow(non_snake_case)]
pub fn BadRequest<E>(err: E) -> Error
where
    E: 'static + fmt::Display,
{
    Error::BadRequest(format!("{}", err))
}

#[allow(non_snake_case)]
pub fn InternalServerError<E>(err: E) -> Error
where
    E: 'static + fmt::Debug,
{
    Error::InternalServerError(format!("{:?}", err))
}
