use actix_http::PayloadStream;
use actix_web::{web, HttpResponse, Scope};
use chrono::prelude::*;
use diesel::prelude::*;
use futures::Future;
use uuid::Uuid;

use super::types::*;
use crate::prelude::{error, utils, Database, Error};

pub fn api(path: &str) -> Scope<PayloadStream> {
    web::scope(path).service(
        web::resource("/register").route(web::post().to_async(register_user)),
    )
}

fn register_user(
    db: web::Data<Database<PgConnection>>,
    register_user: web::Json<RegisterUser>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    debug!("register user: {:?}", &register_user);

    db.transaction(move |conn| {
        use hamster_db::users::dsl::*;

        let register_user = register_user.into_inner();

        if register_user.password != register_user.confirm_password {
            return Err(error::BadRequest("两次输入的密码不匹配"));
        }

        let user_id = Uuid::new_v4();
        let hashed_password = utils::hash_password(&register_user.password)
            .map_err(error::BadRequest)?;
        let random_avatar_url = utils::random_avatar_url();
        let now = Utc::now();

        diesel::insert_into(users)
            .values((
                id.eq(&user_id),
                email.eq(&register_user.email),
                password.eq(&hashed_password),
                avatar_url.eq(&random_avatar_url),
                nickname.eq(&register_user.email),
                is_verified.eq(false),
                created_at.eq(&now),
                updated_at.eq(&now),
            ))
            .execute(conn)?;

        Ok(())
    })
    .from_err()
    .map(move |_| HttpResponse::Created().finish())
}
