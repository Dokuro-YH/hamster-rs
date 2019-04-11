use std::fmt;

use actix_web::{HttpResponse, ResponseError};

use hamster_db::DbError;

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "{}", _0)]
    BadRequest(String),

    #[display(fmt = "r2d2 pool error: {}", _0)]
    R2D2Error(r2d2::Error),

    #[display(fmt = "diesel error: {}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "block execute timeout")]
    Timeout,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        use Error::*;

        match *self {
            BadRequest(ref message) => {
                let payload = json!({ "message": format!("{}", message) });
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

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DieselError(err)
    }
}

impl From<DbError<Error>> for Error {
    fn from(err: DbError<Error>) -> Self {
        match err {
            DbError::Error(err) => err,
            DbError::R2D2Error(err) => Error::R2D2Error(err),
            DbError::Timeout => Error::Timeout,
        }
    }
}

#[allow(non_snake_case)]
pub fn BadRequest<E>(err: E) -> Error
where
    E: 'static + fmt::Display,
{
    Error::BadRequest(format!("{}", err))
}
