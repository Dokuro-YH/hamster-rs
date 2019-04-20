use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::*;
use crate::types::{Group, GroupMembership, GroupMembershipType, NewGroup};

pub fn get_all(conn: &PgConnection) -> Result<Vec<Group>> {
    use crate::schema::groups::dsl::*;

    let result = groups.load::<Group>(conn)?;

    Ok(result)
}

pub fn get_or_create(conn: &PgConnection, name: &str) -> Result<Group> {
    use crate::schema::groups::dsl::*;

    match groups
        .filter(display_name.eq(name))
        .first::<Group>(conn)
        .optional()?
    {
        Some(group) => Ok(group),
        None => create(
            conn,
            NewGroup {
                display_name: name,
                description: None,
            },
        ),
    }
}

pub fn create(conn: &PgConnection, new_group: NewGroup) -> Result<Group> {
    use crate::schema::groups::dsl::*;

    let group_id = Uuid::new_v4();
    let now = Utc::now();
    let result = diesel::insert_into(groups)
        .values((
            id.eq(&group_id),
            display_name.eq(&new_group.display_name),
            description.eq(&new_group.description),
            created_at.eq(&now),
            updated_at.eq(&now),
        ))
        .get_result::<Group>(conn)?;

    Ok(result)
}

pub fn update_desc(
    conn: &PgConnection,
    group_id: &Uuid,
    desc: &str,
) -> Result<()> {
    use crate::schema::groups::dsl::*;

    let _ = diesel::update(groups.find(group_id))
        .set((description.eq(desc), updated_at.eq(Utc::now())))
        .execute(conn)?;

    Ok(())
}

pub fn add_member(
    conn: &PgConnection,
    group_id: &Uuid,
    member_id: &Uuid,
    member_type: GroupMembershipType,
) -> Result<GroupMembership> {
    use crate::schema::group_membership;

    let result = diesel::insert_into(group_membership::table)
        .values((
            group_membership::group_id.eq(&group_id),
            group_membership::member_id.eq(&member_id),
            group_membership::member_type.eq(&member_type),
        ))
        .get_result(conn)?;

    Ok(result)
}
