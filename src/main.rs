#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

mod api;
mod auth;
mod bootstrap;
mod db;
mod error;
mod schema;
mod utils;

#[cfg(test)]
mod test_helpers;

use std::{env, time};

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use failure::Error;

use crate::auth::middleware::{
    AuthenticationService, CookieAuthenticationBackend,
};

static AUTH_SIGNING_KEY: &[u8] = &[0; 32];

fn main() -> Result<(), Error> {
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
        let domain =
            env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
        let auth_backend = CookieAuthenticationBackend::new(AUTH_SIGNING_KEY)
            .name("hamster-auth")
            .path("/")
            .domain(domain.clone())
            .max_age(3600)
            .secure(false);

        App::new()
            .data(db.clone())
            .wrap(AuthenticationService::new(auth_backend))
            .wrap(Logger::default())
            .service(api::service("/api"))
    };

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let address = format!("127.0.0.1:{}", &port);
    let server = HttpServer::new(app).bind(&address)?;

    info!("Server listen on http:://{}", &address);

    server.run()?;

    Ok(())
}
