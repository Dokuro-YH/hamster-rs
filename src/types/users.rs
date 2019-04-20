use chrono::prelude::*;
use uuid::Uuid;

use crate::schema::users;

#[derive(
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Insertable,
    AsChangeset,
    Identifiable,
    Queryable,
)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub nickname: String,
    pub avatar_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub nickname: &'a str,
    pub avatar_url: Option<&'a str>,
}
