use actix_web::{web, Error, HttpResponse, Scope};
use futures::Future;

use crate::db::Database;
use crate::error::Result;
use crate::groups;

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(web::resource("").route(web::get().to_async(get_groups)))
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
    .map(move |groups| HttpResponse::Ok().json(groups))
}
