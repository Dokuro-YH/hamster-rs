mod error;

pub use self::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;
