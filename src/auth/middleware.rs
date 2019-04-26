use std::cell::RefCell;
use std::rc::Rc;

use actix_service::{Service, Transform};
use actix_web::cookie::{Cookie, CookieJar, Key, SameSite};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{self, HeaderValue};
use actix_web::HttpMessage;
use futures::future::{self, Either, FutureResult};
use futures::{Future, IntoFuture, Poll};
use time::Duration;

use super::{Authentication, AuthenticationManager};
use crate::error::{Error, ErrorKind, Result, ResultExt};

/// Authentication storage backend definition
pub trait AuthenticationBackend: Sized + 'static {
    type Future: IntoFuture<Item = Option<Authentication>, Error = Error>;

    fn load(&self, req: &mut ServiceRequest) -> Self::Future;

    fn store<B>(
        &self,
        changed: bool,
        authentication: Option<Authentication>,
        res: &mut ServiceResponse<B>,
    ) -> Self::Future;
}

pub struct AuthenticationService<T> {
    backend: Rc<T>,
}

impl<T> AuthenticationService<T> {
    pub fn new(backend: T) -> Self {
        AuthenticationService {
            backend: Rc::new(backend),
        }
    }
}

impl<S, T, B> Transform<S> for AuthenticationService<T>
where
    B: 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>
        + 'static,
    S::Future: 'static,
    S::Error: 'static,
    T: AuthenticationBackend,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S, T>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(AuthenticationMiddleware {
            backend: self.backend.clone(),
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct AuthenticationMiddleware<S, T> {
    service: Rc<RefCell<S>>,
    backend: Rc<T>,
}

impl<S, T, B> Service for AuthenticationMiddleware<S, T>
where
    B: 'static,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>>
        + 'static,
    S::Future: 'static,
    S::Error: 'static,
    T: AuthenticationBackend,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.borrow_mut().poll_ready()
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let backend = self.backend.clone();

        Box::new(self.backend.load(&mut req).into_future().then(move |res| {
            match res {
                Ok(authentication) => {
                    AuthenticationManager::set_authentication(
                        authentication,
                        &mut req,
                    );

                    Either::A(srv.borrow_mut().call(req).and_then(
                        move |mut res| {
                            let (changed, a) =
                                AuthenticationManager::get_changed(&mut res);
                            backend
                                .store(changed, a, &mut res)
                                .into_future()
                                .then(move |r| match r {
                                    Ok(_) => Ok(res),
                                    Err(e) => Ok(res.error_response(e)),
                                })
                        },
                    ))
                }
                Err(err) => Either::B(future::ok(req.error_response(err))),
            }
        }))
    }
}

struct CookieAuthenticationInner {
    key: Key,
    name: String,
    path: String,
    domain: Option<String>,
    secure: bool,
    max_age: Option<Duration>,
    same_site: Option<SameSite>,
}

impl CookieAuthenticationInner {
    fn new(key: &[u8]) -> CookieAuthenticationInner {
        CookieAuthenticationInner {
            key: Key::from_master(key),
            name: "hamster-auth".to_owned(),
            path: "/".to_owned(),
            domain: None,
            secure: false,
            max_age: None,
            same_site: None,
        }
    }

    fn set_cookie<B>(
        &self,
        resp: &mut ServiceResponse<B>,
        a: Option<&Authentication>,
    ) -> Result<()> {
        let some = a.is_some();
        let value = match a {
            Some(authentication) => serde_json::to_string(authentication)
                .context(ErrorKind::SerializeJsonError)?,
            None => String::new(),
        };
        let mut cookie = Cookie::new(self.name.clone(), value);
        cookie.set_path(self.path.clone());
        cookie.set_secure(self.secure);
        cookie.set_http_only(true);

        if let Some(ref domain) = self.domain {
            cookie.set_domain(domain.clone());
        }

        if let Some(max_age) = self.max_age {
            cookie.set_max_age(max_age);
        }

        if let Some(same_site) = self.same_site {
            cookie.set_same_site(same_site);
        }

        let mut jar = CookieJar::new();
        if some {
            jar.private(&self.key).add(cookie);
        } else {
            jar.add_original(cookie.clone());
            jar.private(&self.key).remove(cookie);
        }

        for cookie in jar.delta() {
            let val = HeaderValue::from_str(&cookie.to_string())
                .context(ErrorKind::HttpHeaderFailure)?;
            resp.headers_mut().append(header::SET_COOKIE, val);
        }

        Ok(())
    }

    fn load(&self, req: &ServiceRequest) -> Result<Option<Authentication>> {
        if let Ok(cookies) = req.cookies() {
            for cookie in cookies.iter() {
                if cookie.name() == self.name {
                    let mut jar = CookieJar::new();
                    jar.add_original(cookie.clone());

                    let cookie_opt = jar.private(&self.key).get(&self.name);
                    if let Some(cookie) = cookie_opt {
                        let authentication =
                            serde_json::from_str(&cookie.value())
                                .context(ErrorKind::DeserializeJsonError)?;
                        return Ok(Some(authentication));
                    }
                }
            }
        }
        Ok(None)
    }
}

pub struct CookieAuthenticationBackend(Rc<CookieAuthenticationInner>);

impl CookieAuthenticationBackend {
    pub fn new(key: &[u8]) -> CookieAuthenticationBackend {
        CookieAuthenticationBackend(Rc::new(CookieAuthenticationInner::new(
            key,
        )))
    }

    pub fn path<S: Into<String>>(
        mut self,
        value: S,
    ) -> CookieAuthenticationBackend {
        Rc::get_mut(&mut self.0).unwrap().path = value.into();
        self
    }

    pub fn name<S: Into<String>>(
        mut self,
        value: S,
    ) -> CookieAuthenticationBackend {
        Rc::get_mut(&mut self.0).unwrap().name = value.into();
        self
    }

    pub fn domain<S: Into<String>>(
        mut self,
        value: S,
    ) -> CookieAuthenticationBackend {
        Rc::get_mut(&mut self.0).unwrap().domain = Some(value.into());
        self
    }

    pub fn secure(mut self, value: bool) -> CookieAuthenticationBackend {
        Rc::get_mut(&mut self.0).unwrap().secure = value;
        self
    }

    pub fn max_age(self, seconds: i64) -> CookieAuthenticationBackend {
        self.max_age_time(Duration::seconds(seconds))
    }

    pub fn max_age_time(
        mut self,
        value: Duration,
    ) -> CookieAuthenticationBackend {
        Rc::get_mut(&mut self.0).unwrap().max_age = Some(value);
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        Rc::get_mut(&mut self.0).unwrap().same_site = Some(same_site);
        self
    }
}

impl AuthenticationBackend for CookieAuthenticationBackend {
    type Future = Result<Option<Authentication>>;

    fn load(&self, req: &mut ServiceRequest) -> Self::Future {
        self.0.load(req)
    }

    fn store<B>(
        &self,
        changed: bool,
        authentication: Option<Authentication>,
        res: &mut ServiceResponse<B>,
    ) -> Self::Future {
        if changed {
            let _ = self.0.set_cookie(res, authentication.as_ref());
        }
        Ok(authentication)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test::{self, TestRequest};
    use actix_web::{web, App, HttpResponse};

    #[test]
    fn test_cookie_authentication() {
        let mut app = test::init_service(
            App::new()
                .wrap(AuthenticationService::new(
                    CookieAuthenticationBackend::new(&[0; 32]).secure(false),
                ))
                .service(web::resource("/").to(|am: AuthenticationManager| {
                    let a = am
                        .authentication()
                        .unwrap_or_else(Authentication::anonymous);

                    a.identity().to_string()
                }))
                .service(web::resource("/login").to(
                    |am: AuthenticationManager| {
                        am.remember(Authentication::new(
                            "admin",
                            vec!["admin".to_string()],
                        ));
                        HttpResponse::Ok()
                    },
                ))
                .service(web::resource("/logout").to(
                    |am: AuthenticationManager| {
                        am.forget();
                        HttpResponse::Ok()
                    },
                )),
        );

        let resp = test::call_service(
            &mut app,
            TestRequest::with_uri("/").to_request(),
        );
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
