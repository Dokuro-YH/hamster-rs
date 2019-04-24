pub use actix_web::error::*;
use actix_web::ResponseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Display, From)]
pub enum BootstrapError {
    #[display(fmt = "file not found: {}", _0)]
    FileNotFound(std::io::Error),
    #[display(fmt = "toml parse error: {}", _0)]
    TomlParse(toml::de::Error),
    #[display(fmt = "database connection error: {}", _0)]
    Connection(diesel::result::ConnectionError),
    #[display(fmt = "user info parse: {}", _0)]
    UserInfoParse(String),
    #[display(fmt = "db error: {}", _0)]
    DbError(DbError),
}

impl ResponseError for BootstrapError {}

#[derive(Debug, Display, From)]
pub enum DbError {
    #[display(fmt = "block execute timeout")]
    Timeout,

    #[display(fmt = "diesel error: {}", _0)]
    DieselError(diesel::result::Error),

    #[display(fmt = "r2d2 pool error: {}", _0)]
    R2D2Error(r2d2::Error),

    #[display(fmt = "bcrypt error: {}", _0)]
    BcryptError(bcrypt::BcryptError),
}

impl ResponseError for DbError {}
