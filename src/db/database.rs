use std::time::Duration;

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

use crate::error::{ErrorKind, Result, ResultExt};

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

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
    pub fn conn(&self) -> Result<Conn> {
        let pool = self.pool.clone();

        Ok(pool.get().context(ErrorKind::DbPoolError)?)
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
