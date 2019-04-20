pub mod database;
pub mod error;

pub use self::database::{Database, DatabaseBuilder};
pub use self::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
