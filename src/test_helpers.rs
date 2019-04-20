use diesel::prelude::*;

pub fn connection() -> PgConnection {
    let database_url =
        dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url).unwrap();
    conn.begin_test_transaction().unwrap();
    conn
}
