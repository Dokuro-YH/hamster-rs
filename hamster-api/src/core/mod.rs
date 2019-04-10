pub mod db;
pub mod error;
pub mod utils;

pub use self::db::*;
pub use self::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
