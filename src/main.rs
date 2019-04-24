#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate diesel;

mod api;
mod bootstrap;
mod db;
mod error;
mod schema;
mod types;
mod utils;

#[cfg(test)]
mod test_helpers;

use std::{env, time};

use actix_web::{middleware, App, HttpServer};

fn main() -> error::Result<()> {
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

    bootstrap::run(&database_url, "bootstrap.toml")?;
    let app = move || {
        App::new()
            .data(db.clone())
            .wrap(middleware::Logger::default())
            .service(api::service("/api"))
    };

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let address = format!("127.0.0.1:{}", &port);
    let server = HttpServer::new(app).bind(&address)?;

    info!("Server listen on http:://{}", &address);

    server.run()?;

    Ok(())
}
