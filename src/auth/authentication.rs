use std::cell::RefCell;
use std::collections::HashSet;
use std::mem;
use std::rc::Rc;

use actix_web::dev::{Extensions, Payload, ServiceRequest, ServiceResponse};
use actix_web::{FromRequest, HttpMessage, HttpRequest};

use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Authentication {
    identity: String,
    authorities: HashSet<String>,
}

impl Authentication {
    pub fn new<I: Into<String>, A: IntoIterator<Item = String>>(
        identity: I,
        authorities: A,
    ) -> Self {
        Authentication {
            identity: identity.into(),
            authorities: authorities.into_iter().collect(),
        }
    }

    pub fn anonymous() -> Self {
        Self::new("anonymous", vec!["anonymous".to_string()])
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

pub struct AuthenticationManager(Rc<RefCell<AuthenticationManagerInner>>);

impl AuthenticationManager {
    pub fn authentication(&self) -> Option<Authentication> {
        let inner = self.0.borrow();
        inner.authentication.clone()
    }

    pub fn remember(&self, a: Authentication) {
        let mut inner = self.0.borrow_mut();
        inner.authentication = Some(a);
        inner.changed = true;
    }

    pub fn forget(&self) {
        let mut inner = self.0.borrow_mut();
        inner.authentication = None;
        inner.changed = true;
    }

    fn get_manager(extensions: &mut Extensions) -> Self {
        if let Some(inner) =
            extensions.get::<Rc<RefCell<AuthenticationManagerInner>>>()
        {
            return AuthenticationManager(inner.clone());
        }

        let inner = Rc::new(RefCell::new(AuthenticationManagerInner::new()));
        extensions.insert(inner.clone());
        AuthenticationManager(inner)
    }

    pub(crate) fn set_authentication(
        a: Option<Authentication>,
        req: &mut ServiceRequest,
    ) {
        let am = AuthenticationManager::get_manager(&mut req.extensions_mut());
        let mut inner = am.0.borrow_mut();
        inner.authentication = a;
    }

    pub(crate) fn get_changed<B>(
        res: &mut ServiceResponse<B>,
    ) -> (bool, Option<Authentication>) {
        if let Some(inner) = res
            .request()
            .extensions()
            .get::<Rc<RefCell<AuthenticationManagerInner>>>()
        {
            let mut inner = inner.borrow_mut();
            let changed = mem::replace(&mut inner.changed, false);
            let authentication = mem::replace(&mut inner.authentication, None);
            (changed, authentication)
        } else {
            (false, None)
        }
    }
}

struct AuthenticationManagerInner {
    changed: bool,
    authentication: Option<Authentication>,
}

impl AuthenticationManagerInner {
    pub fn new() -> Self {
        AuthenticationManagerInner {
            changed: false,
            authentication: None,
        }
    }
}

/// Extractor implementation for AuthenticationManager type.
impl FromRequest for AuthenticationManager {
    type Config = ();
    type Error = ();
    type Future = Result<AuthenticationManager, ()>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        Ok(AuthenticationManager::get_manager(
            &mut req.extensions_mut(),
        ))
    }
}

impl FromRequest for Authentication {
    type Config = ();
    type Error = Error;
    type Future = Result<Authentication, Error>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let am = AuthenticationManager::get_manager(&mut req.extensions_mut());
        match am.authentication() {
            Some(a) => Ok(a),
            None => Err(ErrorKind::Unauthorized)?,
        }
    }
}
