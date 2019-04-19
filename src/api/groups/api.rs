use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use futures::Future;

use super::types::*;
use crate::prelude::*;

pub fn get_groups(
    db: web::Data<Database>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.transaction(move |conn| {
        use crate::schema::groups::dsl::*;

        let result = groups.load::<Group>(conn)?;

        Ok(result)
    })
    .map(move |groups| HttpResponse::Ok().json(groups))
}
