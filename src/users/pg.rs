use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use super::types::{NewUser, User};
use crate::error::{ErrorKind, Result, ResultExt};
use crate::utils;

pub fn find_all(conn: &PgConnection) -> Result<Vec<User>> {
    use crate::schema::users;

    Ok(users::table.load(conn).context(ErrorKind::DbError)?)
}

pub fn find_by_username(
    conn: &PgConnection,
    username: &str,
) -> Result<Option<User>> {
    use crate::schema::users;

    Ok(users::table
        .filter(users::username.eq(username))
        .first(conn)
        .optional()
        .context(ErrorKind::DbError)?)
}

pub fn create_or_update(
    conn: &PgConnection,
    username: &str,
    nickname: &str,
    password: &str,
) -> Result<User> {
    use crate::schema::users;

    let result = match users::table
        .filter(users::username.eq(&username))
        .first::<User>(conn)
        .optional()
        .context(ErrorKind::DbError)?
    {
        None => create(
            conn,
            NewUser {
                username,
                password,
                nickname,
                avatar_url: None,
            },
        )?,
        Some(user) => change_user(conn, user, nickname, password)?,
    };

    Ok(result)
}

pub fn create(conn: &PgConnection, new_user: NewUser) -> Result<User> {
    use crate::schema::users;

    let user_id = Uuid::new_v4();
    let avatar_url = new_user
        .avatar_url
        .map(ToString::to_string)
        .unwrap_or_else(utils::random_avatar);
    let now = Utc::now();

    Ok(diesel::insert_into(users::table)
        .values((
            users::id.eq(&user_id),
            users::username.eq(&new_user.username),
            users::password.eq(&new_user.password),
            users::nickname.eq(&new_user.nickname),
            users::avatar_url.eq(&avatar_url),
            users::created_at.eq(&now),
            users::updated_at.eq(&now),
        ))
        .get_result(conn)
        .context(ErrorKind::DbError)?)
}

fn change_user(
    conn: &PgConnection,
    mut user: User,
    nickname: &str,
    password: &str,
) -> Result<User> {
    user.nickname = nickname.to_string();

    user.password = password.to_string();

    Ok(user
        .save_changes::<User>(conn)
        .context(ErrorKind::DbError)?)
}
