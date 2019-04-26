use actix_web::{web, Error, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

use crate::db::{
    groups,
    users::{self, NewUser},
    Database,
};
use crate::error::Result;

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(
            web::resource("")
                .route(web::get().to_async(get_users))
                .route(web::post().to_async(add_user)),
        )
        .service(
            web::resource("/{user_id}").route(web::delete().to_async(del_user)),
        )
}

fn get_users(
    db: web::Data<Database>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || -> Result<_> {
        let conn = db.conn()?;
        let result = users::find_all(&conn)?;
        Ok(result)
    })
    .from_err()
    .map(|res| HttpResponse::Ok().json(res))
}

fn add_user(
    db: web::Data<Database>,
    new: web::Json<NewUser>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let new = new.into_inner();
    web::block(move || -> Result<_> {
        let conn = db.conn()?;
        let result = users::create(&conn, new)?;
        Ok(result)
    })
    .from_err()
    .map(|res| HttpResponse::Created().json(res))
}

fn del_user(
    db: web::Data<Database>,
    user_id: web::Path<Uuid>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || -> Result<_> {
        db.transaction(|conn| {
            groups::del_members_by_member_id(conn, &user_id)?;
            users::del_by_id(conn, &user_id)?;

            Ok(())
        })
    })
    .from_err()
    .map(|_| HttpResponse::NoContent().finish())
}
