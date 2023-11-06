use connection::*;

use bb8;
use bb8_postgres;
use futures::future;
use futures::prelude::*;
use tokio_postgres;

#[derive(Clone, Debug)]
pub struct Pool {
    inner: bb8::Pool<bb8_postgres::PostgresConnectionManager>,
}

impl Pool {
    pub fn run<F, U, T, E>(&self, f: F) -> impl Future<Item = T, Error = E>
    where
        F: FnOnce(BoxedConnection<E>) -> U + 'static,
        U: IntoFuture<Item = (T, BoxedConnection<E>), Error = (E, BoxedConnection<E>)> + 'static,
        T: 'static,
        E: From<tokio_postgres::Error> + 'static,
    {
        self.inner.run(move |conn| {
            conn.transaction().map_err(|(e, conn)| (E::from(e), conn)).and_then(|t| {
                f(Box::new(t) as BoxedConnection<E>)
                    .into_future()
                    .then(|res| match res {
                        Ok((v, conn)) => Box::new(conn.commit2().map(move |(_, conn)| (v, conn)))
                            as Box<Future<Item = (T, BoxedConnection<E>), Error = (E, BoxedConnection<E>)>>,
                        Err((e, conn)) => Box::new(conn.rollback2().and_then(move |(_, conn)| future::err((e, conn)))),
                    })
                    .map(|(v, conn)| (v, conn.unwrap_tokio_postgres()))
                    .map_err(|(e, conn)| (e, conn.unwrap_tokio_postgres()))
            })
        })
    }
}

impl From<bb8::Pool<bb8_postgres::PostgresConnectionManager>> for Pool {
    fn from(v: bb8::Pool<bb8_postgres::PostgresConnectionManager>) -> Self {
        Self { inner: v }
    }
}
