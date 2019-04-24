use actix_web::{web, HttpResponse, Scope};
use futures::Future;

use crate::db::{groups, Database};
use crate::error::Error;

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(web::resource("").route(web::get().to_async(get_groups)))
}

fn get_groups(
    db: web::Data<Database>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.transaction(move |conn| groups::get_all(conn))
        .map(move |groups| HttpResponse::Ok().json(groups))
        .from_err()
}
