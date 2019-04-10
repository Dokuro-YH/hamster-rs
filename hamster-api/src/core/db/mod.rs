use diesel::pg::PgConnection;

mod database;

pub type Database = self::database::Database<PgConnection>;
