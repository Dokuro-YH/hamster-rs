use std::io;

use chrono::prelude::*;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use uuid::Uuid;

use crate::schema::groups;

#[derive(Debug, PartialEq, Deserialize, Serialize, Insertable, Queryable)]
#[table_name = "groups"]
pub struct Group {
    pub id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewGroup {
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateGroup {
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Queryable)]
pub struct GroupMembership {
    pub group_id: Uuid,
    pub member_id: Uuid,
    pub member_type: GroupMembershipType,
    pub added: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, FromSqlRow, AsExpression)]
#[sql_type = "Text"]
pub enum GroupMembershipType {
    User,
    Group,
}

impl ToSql<Text, Pg> for GroupMembershipType {
    fn to_sql<W: io::Write>(
        &self,
        out: &mut Output<W, Pg>,
    ) -> serialize::Result {
        use self::GroupMembershipType::*;
        match *self {
            User => out.write_all(b"user")?,
            Group => out.write_all(b"group")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for GroupMembershipType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        use self::GroupMembershipType::*;
        match not_none!(bytes) {
            b"user" => Ok(User),
            b"group" => Ok(Group),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}
