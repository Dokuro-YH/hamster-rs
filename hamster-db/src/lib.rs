#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate diesel;

mod database;
mod error;
mod schema;

pub use database::{Database, DatabaseBuilder};
pub use error::DbError;
pub use schema::*;
