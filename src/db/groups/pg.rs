use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use super::types::{
    Group, GroupMembership, GroupMembershipType, NewGroup, UpdateGroup,
};
use crate::db::{self, Conn};
use crate::error::{ErrorKind, Result, ResultExt};

pub fn find_all(conn: &Conn) -> Result<Vec<Group>> {
    use crate::schema::groups::dsl::*;

    let result = groups.load::<Group>(conn).context(ErrorKind::DbError)?;

    Ok(result)
}

pub fn find_by_name(conn: &Conn, name: &str) -> Result<Option<Group>> {
    use crate::schema::groups::dsl::*;

    Ok(groups
        .filter(display_name.eq(name))
        .first::<Group>(conn)
        .optional()
        .context(ErrorKind::DbError)?)
}

pub fn find_by_member_id(conn: &Conn, member_id: &Uuid) -> Result<Vec<Group>> {
    let mut result = Vec::new();

    let mut member_ids = vec![*member_id];
    let mut groups = find_groups_by_member_ids(conn, &member_ids)?;

    while !groups.is_empty() {
        member_ids = groups.iter().map(|g| g.id).collect();
        result.append(&mut groups);
        groups = find_groups_by_member_ids(conn, &member_ids)?;
    }

    Ok(result)
}

pub fn get_or_create(conn: &Conn, name: &str) -> Result<Group> {
    use crate::schema::groups::dsl::*;

    let group_opt = groups
        .filter(display_name.eq(name))
        .first::<Group>(conn)
        .optional()
        .context(ErrorKind::DbError)?;

    match group_opt {
        Some(group) => Ok(group),
        None => create(
            conn,
            NewGroup {
                display_name: name.to_string(),
                description: None,
            },
        ),
    }
}

pub fn create(conn: &Conn, new_group: NewGroup) -> Result<Group> {
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
        .get_result::<Group>(conn)
        .context(ErrorKind::DbError)?;

    Ok(result)
}

pub fn update(
    conn: &Conn,
    group_id: &Uuid,
    update: UpdateGroup,
) -> Result<usize> {
    use crate::schema::groups::dsl::*;

    Ok(diesel::update(groups.find(group_id))
        .set((
            display_name.eq(&update.display_name),
            description.eq(&update.description),
            updated_at.eq(Utc::now()),
        ))
        .execute(conn)
        .context(ErrorKind::DbError)?)
}

pub fn update_desc(conn: &Conn, group_id: &Uuid, desc: &str) -> Result<usize> {
    use crate::schema::groups::dsl::*;

    Ok(diesel::update(groups.find(group_id))
        .set((description.eq(desc), updated_at.eq(Utc::now())))
        .execute(conn)
        .context(ErrorKind::DbError)?)
}

pub fn del_by_id(conn: &Conn, group_id: &Uuid) -> Result<usize> {
    use crate::schema::groups;

    Ok(diesel::delete(groups::table)
        .filter(groups::id.eq(group_id))
        .execute(conn)
        .context(ErrorKind::DbError)?)
}

pub fn add_member(
    conn: &Conn,
    group_id: &Uuid,
    member_id: &Uuid,
    member_type: GroupMembershipType,
) -> Result<GroupMembership> {
    use crate::schema::group_membership;

    let member = group_membership::table
        .find((group_id, member_id))
        .first(conn)
        .optional()
        .context(ErrorKind::DbError)?;

    if let Some(result) = member {
        Ok(result)
    } else {
        let result = diesel::insert_into(group_membership::table)
            .values((
                group_membership::group_id.eq(&group_id),
                group_membership::member_id.eq(&member_id),
                group_membership::member_type.eq(&member_type),
            ))
            .get_result(conn)
            .context(ErrorKind::DbError)?;

        Ok(result)
    }
}

pub fn del_members_by_group_id(conn: &Conn, group_id: &Uuid) -> Result<usize> {
    use crate::schema::group_membership;

    Ok(diesel::delete(group_membership::table)
        .filter(group_membership::group_id.eq(group_id))
        .execute(conn)
        .context(ErrorKind::DbError)?)
}

pub fn del_members_by_member_id(
    conn: &Conn,
    member_id: &Uuid,
) -> Result<usize> {
    use crate::schema::group_membership;

    Ok(diesel::delete(group_membership::table)
        .filter(group_membership::member_id.eq(member_id))
        .execute(conn)
        .context(ErrorKind::DbError)?)
}

fn find_groups_by_member_ids(
    conn: &Conn,
    member_ids: &[Uuid],
) -> Result<Vec<Group>> {
    use crate::schema::{group_membership, groups};
    use diesel::dsl::any;

    Ok(group_membership::table
        .inner_join(groups::table)
        .select(groups::all_columns)
        .filter(group_membership::member_id.eq(any(member_ids)))
        .load(conn)
        .context(ErrorKind::DbError)?)
}
