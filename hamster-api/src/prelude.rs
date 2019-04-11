use diesel::PgConnection;

pub use crate::error::Error;

pub type Database<C = PgConnection> = hamster_db::Database<C>;

pub mod error {
    pub use crate::error::*;
}

pub mod utils {
    pub use crate::utils::*;
}
