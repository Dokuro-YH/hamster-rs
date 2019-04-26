use actix_web::{web, Error, HttpResponse, Scope};
use futures::Future;

use crate::auth::{Authentication, AuthenticationManager};
use crate::db::{groups, users, Database};
use crate::error::{ErrorKind, Result};
use crate::utils;

#[derive(Debug, Deserialize)]
struct AuthData {
    username: String,
    password: String,
}

pub fn service(path: &str) -> Scope {
    web::scope(path).service(
        web::resource("")
            .route(web::get().to(userinfo))
            .route(web::post().to_async(login))
            .route(web::delete().to(logout)),
    )
}

fn userinfo(a: Authentication) -> HttpResponse {
    let identity = a.identity();
    HttpResponse::Ok().json(identity)
}

fn login(
    auth_data: web::Json<AuthData>,
    db: web::Data<Database>,
    am: AuthenticationManager,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let auth_data = auth_data.into_inner();

    web::block(move || -> Result<Authentication> {
        let conn = db.conn()?;
        if let Some(user) = users::find_by_username(&conn, &auth_data.username)?
        {
            let verified_password =
                utils::verify_password(&auth_data.password, &user.password)?;

            if verified_password {
                let groups = groups::find_by_member_id(&conn, &user.id)?;
                let authorities = groups.into_iter().map(|g| g.display_name);
                let identity = user.id.simple().to_string();
                let authentication = Authentication::new(identity, authorities);

                return Ok(authentication);
            }
        }

        Err(ErrorKind::Unauthorized)?
    })
    .from_err()
    .map(move |a| {
        am.remember(a);
        HttpResponse::Ok().finish()
    })
}

fn logout(am: AuthenticationManager) -> HttpResponse {
    am.forget();

    HttpResponse::Ok().finish()
}
