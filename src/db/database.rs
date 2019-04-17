use std::time::Duration;

use actix_web::{error::BlockingError, web};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, PgConnection,
};
use futures::future::Future;

use crate::core::Error;

pub struct Database {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}

impl Database {
    #[inline]
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder {
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
            pool_idle_timeout: None,
        }
    }

    #[inline]
    pub fn transaction<F, R>(
        &self,
        f: F,
    ) -> impl Future<Item = R, Error = Error>
    where
        F: 'static + FnOnce(&PgConnection) -> Result<R, Error> + Send,
        R: 'static + Send,
    {
        self.get(move |conn| conn.transaction(move || f(conn)))
    }

    pub fn get<F, R>(&self, f: F) -> impl Future<Item = R, Error = Error>
    where
        F: 'static + FnOnce(&PgConnection) -> Result<R, Error> + Send,
        R: 'static + Send,
    {
        let pool = self.pool.clone();

        web::block(move || match pool.get() {
            Ok(conn) => Ok((f)(&*conn)),
            Err(err) => Err(err),
        })
        .then(|res| match res {
            Ok(res) => match res {
                Ok(value) => Ok(value),
                Err(err) => Err(err),
            },
            Err(err) => match err {
                BlockingError::Canceled => Err(Error::Timeout),
                BlockingError::Error(err) => Err(Error::R2D2Error(err)),
            },
        })
    }
}

pub struct DatabaseBuilder {
    pub pool_max_size: Option<u32>,
    pub pool_min_idle: Option<u32>,
    pub pool_max_lifetime: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
}

impl DatabaseBuilder {
    #[inline]
    pub fn pool_max_size(&mut self, max_size: u32) -> &mut Self {
        self.pool_max_size = Some(max_size);
        self
    }

    #[inline]
    pub fn pool_min_idle(&mut self, min_idle: Option<u32>) -> &mut Self {
        self.pool_min_idle = min_idle;
        self
    }

    #[inline]
    pub fn pool_max_lifetime(
        &mut self,
        max_lifetime: Option<Duration>,
    ) -> &mut Self {
        self.pool_max_lifetime = max_lifetime;
        self
    }

    #[inline]
    pub fn pool_idle_timeout(
        &mut self,
        idle_timeout: Option<Duration>,
    ) -> &mut Self {
        self.pool_idle_timeout = idle_timeout;
        self
    }

    pub fn open(&mut self, url: &str) -> Database {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let mut p = Pool::builder();

        if let Some(max_size) = self.pool_max_size {
            p = p.max_size(max_size);
        }

        if let Some(min_idle) = self.pool_min_idle {
            p = p.min_idle(Some(min_idle));
        }

        if let Some(max_lifetime) = self.pool_max_lifetime {
            p = p.max_lifetime(Some(max_lifetime));
        }

        if let Some(idle_timeout) = self.pool_idle_timeout {
            p = p.idle_timeout(Some(idle_timeout));
        }

        let pool = p.build_unchecked(manager);

        Database { pool }
    }
}
