mod groups;

use actix_files::Files;
use actix_web::{web, Scope};

pub fn service(path: &str) -> Scope {
    web::scope(path)
        .service(groups::service("/groups"))
        .service(Files::new("/images", "images/"))
}
