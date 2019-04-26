use actix_web::{web, Error, HttpResponse, Scope};
use futures::Future;
use uuid::Uuid;

use crate::db::{
    groups::{self, NewGroup, UpdateGroup},
    Database,
};
use crate::error::Result;

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(
            web::resource("")
                .route(web::get().to_async(get_groups))
                .route(web::post().to_async(add_group)),
        )
        .service(
            web::resource("/{group_id}")
                .route(web::put().to_async(update_group))
                .route(web::delete().to_async(del_group)),
        )
}

fn get_groups(
    db: web::Data<Database>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || -> Result<_> {
        let conn = db.conn()?;
        let result = groups::find_all(&conn)?;
        Ok(result)
    })
    .from_err()
    .map(|res| HttpResponse::Ok().json(res))
}

fn add_group(
    db: web::Data<Database>,
    new: web::Json<NewGroup>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let new = new.into_inner();
    web::block(move || -> Result<_> {
        let conn = db.conn()?;
        let result = groups::create(&conn, new)?;
        Ok(result)
    })
    .from_err()
    .map(|res| HttpResponse::Created().json(res))
}

fn update_group(
    db: web::Data<Database>,
    group_id: web::Path<Uuid>,
    update: web::Json<UpdateGroup>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let update = update.into_inner();
    web::block(move || -> Result<_> {
        let conn = db.conn()?;
        let result = groups::update(&conn, &group_id, update)?;
        Ok(result)
    })
    .from_err()
    .map(|res| HttpResponse::Ok().json(res))
}

fn del_group(
    db: web::Data<Database>,
    group_id: web::Path<Uuid>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || -> Result<_> {
        db.transaction(|conn| {
            groups::del_members_by_member_id(conn, &group_id)?;
            groups::del_members_by_group_id(conn, &group_id)?;
            groups::del_by_id(conn, &group_id)?;

            Ok(())
        })
    })
    .from_err()
    .map(|_| HttpResponse::NoContent().finish())
}
