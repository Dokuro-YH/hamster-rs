use std::fmt;

use actix_web::ResponseError;

#[derive(Debug, Display)]
pub enum DbError<E>
where
    E: 'static + fmt::Debug + fmt::Display + Send + Sync,
{
    #[display(fmt = "r2d2 pool error: {}", _0)]
    R2D2Error(r2d2::Error),

    #[display(fmt = "execute timeout")]
    Timeout,

    #[display(fmt = "{}", _0)]
    Error(E),
}

impl<E> ResponseError for DbError<E> where E: 'static + fmt::Debug + fmt::Display + Send + Sync {}
