use std::fmt::{self, Display};

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
pub use failure::ResultExt;
use failure::{Backtrace, Context, Fail};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Fail)]
pub enum ErrorKind {
    #[fail(display = "Failed to get connection")]
    DbPoolError,

    #[fail(display = "Database access error")]
    DbError,

    #[fail(display = "Application bootstrap error")]
    BootstrapError,

    #[fail(display = "Unauthorized")]
    Unauthorized,

    #[fail(display = "Serialize json error")]
    SerializeJsonError,

    #[fail(display = "Deserialize json error")]
    DeserializeJsonError,

    #[fail(display = "Invalid http header value")]
    HttpHeaderFailure,

    #[fail(display = "Failed to bcrypt password")]
    HashPasswordFailure,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        use self::ErrorKind::*;

        match self.kind() {
            Unauthorized => HttpResponse::new(StatusCode::UNAUTHORIZED),
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}
