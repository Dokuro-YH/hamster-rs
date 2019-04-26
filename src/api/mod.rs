mod auth;
mod groups;
mod users;

use actix_files::Files;
use actix_web::{web, Scope};

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(auth::service("/auth"))
        .service(groups::service("/groups"))
        .service(users::service("/users"))
        .service(Files::new("/images", "./images"))
}
