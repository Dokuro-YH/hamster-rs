use chrono::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::prelude::*;
use crate::types::{NewUser, User};
use crate::utils;

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
        .optional()?
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
    let hashed_password = utils::hash_password(new_user.password)?;
    let avatar_url = utils::random_avatar();
    let now = Utc::now();
    let user = diesel::insert_into(users::table)
        .values((
            users::id.eq(&user_id),
            users::username.eq(&new_user.username),
            users::password.eq(&hashed_password),
            users::nickname.eq(&new_user.nickname),
            users::avatar_url.eq(&avatar_url),
            users::created_at.eq(&now),
            users::updated_at.eq(&now),
        ))
        .get_result(conn)?;

    Ok(user)
}

fn change_user(
    conn: &PgConnection,
    mut user: User,
    nickname: &str,
    password: &str,
) -> Result<User> {
    user.nickname = nickname.to_string();

    user.password = password.to_string();

    user = user.save_changes::<User>(conn)?;

    Ok(user)
}
