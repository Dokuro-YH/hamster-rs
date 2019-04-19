mod api;
mod types;

use actix_web::{web, Scope};

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(web::resource("").route(web::get().to_async(api::get_groups)))
}
