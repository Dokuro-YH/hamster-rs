use std::{fmt, marker::PhantomData, time::Duration};

use actix_web::{error::BlockingError, web};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection,
};
use futures::future::Future;

use crate::DbError;

pub struct Database<C: 'static>
where
    C: Connection,
{
    pub pool: Pool<ConnectionManager<C>>,
}

impl<C> Clone for Database<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}

impl<C> Database<C>
where
    C: Connection,
{
    #[inline]
    pub fn builder() -> DatabaseBuilder<C> {
        DatabaseBuilder {
            phantom: PhantomData,
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
            pool_idle_timeout: None,
        }
    }

    #[inline]
    pub fn transaction<F, R, E>(&self, f: F) -> impl Future<Item = R, Error = DbError<E>>
    where
        F: 'static + FnOnce(&C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + From<diesel::result::Error> + fmt::Display + fmt::Debug + Send + Sync,
    {
        self.get(move |conn| conn.transaction(move || f(conn)))
    }

    pub fn get<F, R, E>(&self, f: F) -> impl Future<Item = R, Error = DbError<E>>
    where
        F: 'static + FnOnce(&C) -> Result<R, E> + Send,
        R: 'static + Send,
        E: 'static + fmt::Display + fmt::Debug + Send + Sync,
    {
        let pool = self.pool.clone();

        web::block(move || match pool.get() {
            Ok(conn) => Ok((f)(&*conn)),
            Err(err) => Err(err),
        })
        .then(|res| match res {
            Ok(res) => match res {
                Ok(value) => Ok(value),
                Err(err) => Err(DbError::Error(err)),
            },
            Err(err) => match err {
                BlockingError::Canceled => Err(DbError::Timeout),
                BlockingError::Error(err) => Err(DbError::R2D2Error(err)),
            },
        })
    }
}

pub struct DatabaseBuilder<C: 'static>
where
    C: Connection,
{
    pub phantom: PhantomData<C>,
    pub pool_max_size: Option<u32>,
    pub pool_min_idle: Option<u32>,
    pub pool_max_lifetime: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
}
impl<C> DatabaseBuilder<C>
where
    C: Connection,
{
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
    pub fn pool_max_lifetime(&mut self, max_lifetime: Option<Duration>) -> &mut Self {
        self.pool_max_lifetime = max_lifetime;
        self
    }

    #[inline]
    pub fn pool_idle_timeout(&mut self, idle_timeout: Option<Duration>) -> &mut Self {
        self.pool_idle_timeout = idle_timeout;
        self
    }

    pub fn open(&mut self, url: impl Into<String>) -> Database<C> {
        let manager = ConnectionManager::<C>::new(url);
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
