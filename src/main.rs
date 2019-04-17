#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate derive_more;

pub mod api;
pub mod core;
pub mod db;
pub mod utils;

use std::{env, io, time};

use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};

fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "hamster=debug,actix_web=info");

    dotenv::dotenv().ok();
    pretty_env_logger::init_timed();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = db::Database::builder()
        .pool_max_size(10)
        .pool_min_idle(Some(0))
        .pool_max_lifetime(Some(time::Duration::from_secs(30 * 60)))
        .pool_idle_timeout(Some(time::Duration::from_secs(10 * 60)))
        .open(&database_url);

    let app = move || {
        App::new()
            .data(db.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api").service(Files::new("/images", "images/")),
            )
    };

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let address = format!("127.0.0.1:{}", &port);
    let server = HttpServer::new(app).bind(&address)?;

    info!("Server listen on http:://{}", &address);

    server.run()
}
