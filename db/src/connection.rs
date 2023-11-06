use futures::future;
use futures::prelude::*;
use futures_state_stream::*;
use std::convert::From;
use tokio_postgres;
use tokio_postgres::rows::Row;
use tokio_postgres::stmt::Statement;
use tokio_postgres::transaction::Transaction;
use tokio_postgres::types::ToSql;

pub type BoxedConnection<E> = Box<Connection<E>>;
pub type ConnectionFuture<T, E> = Box<Future<Item = (T, BoxedConnection<E>), Error = (E, BoxedConnection<E>)>>;

pub trait Connection<E>
where
    E: From<tokio_postgres::Error>,
{
    fn prepare2(self: Box<Self>, query: &str) -> ConnectionFuture<Statement, E>;
    fn query2(
        self: Box<Self>,
        statement: &Statement,
        params: Vec<Box<ToSql>>,
    ) -> Box<StateStream<Item = Row, State = BoxedConnection<E>, Error = E>>;
    fn commit2(self: Box<Self>) -> ConnectionFuture<(), E>;
    fn rollback2(self: Box<Self>) -> ConnectionFuture<(), E>;
    fn unwrap_tokio_postgres(self: Box<Self>) -> tokio_postgres::Connection;
}

impl<E> Connection<E> for Transaction
where
    E: From<tokio_postgres::Error> + 'static,
{
    fn prepare2(self: Box<Self>, query: &str) -> ConnectionFuture<Statement, E> {
        Box::new(
            self.prepare(query)
                .map(|(v, conn)| (v, Box::new(conn) as BoxedConnection<E>))
                .map_err(|(e, conn)| (E::from(e), Box::new(conn) as BoxedConnection<E>)),
        )
    }

    fn query2(
        self: Box<Self>,
        statement: &Statement,
        params: Vec<Box<ToSql>>,
    ) -> Box<StateStream<Item = Row, State = BoxedConnection<E>, Error = E>> {
        Box::new(
            self.query(statement, &params.iter().map(|v| &**v as &ToSql).collect::<Vec<&ToSql>>())
                .map_err(E::from)
                .map_state(|conn| Box::new(conn) as BoxedConnection<E>),
        )
    }

    fn commit2(self: Box<Self>) -> ConnectionFuture<(), E> {
        Box::new(
            self.commit()
                .map(|conn| ((), Box::new(conn) as BoxedConnection<E>))
                .map_err(|(e, conn)| (E::from(e), Box::new(conn) as BoxedConnection<E>)),
        )
    }

    fn rollback2(self: Box<Self>) -> ConnectionFuture<(), E> {
        Box::new(
            self.rollback()
                .map(|conn| ((), Box::new(conn) as BoxedConnection<E>))
                .map_err(|(e, conn)| (E::from(e), Box::new(conn) as BoxedConnection<E>)),
        )
    }

    fn unwrap_tokio_postgres(self: Box<Self>) -> tokio_postgres::Connection {
        unreachable!()
    }
}

impl<E> Connection<E> for tokio_postgres::Connection
where
    E: From<tokio_postgres::Error> + 'static,
{
    fn prepare2(self: Box<Self>, query: &str) -> ConnectionFuture<Statement, E> {
        Box::new(
            self.prepare(query)
                .map(|(v, conn)| (v, Box::new(conn) as BoxedConnection<E>))
                .map_err(|(e, conn)| (E::from(e), Box::new(conn) as BoxedConnection<E>)),
        )
    }

    fn query2(
        self: Box<Self>,
        statement: &Statement,
        params: Vec<Box<ToSql>>,
    ) -> Box<StateStream<Item = Row, State = BoxedConnection<E>, Error = E>> {
        Box::new(
            self.query(statement, &params.iter().map(|v| &**v as &ToSql).collect::<Vec<&ToSql>>())
                .map_err(E::from)
                .map_state(|conn| Box::new(conn) as BoxedConnection<E>),
        )
    }

    fn commit2(self: Box<Self>) -> ConnectionFuture<(), E> {
        Box::new(future::ok(((), self as BoxedConnection<E>)))
    }

    fn rollback2(self: Box<Self>) -> ConnectionFuture<(), E> {
        Box::new(future::ok(((), self as BoxedConnection<E>)))
    }

    fn unwrap_tokio_postgres(self: Box<Self>) -> tokio_postgres::Connection {
        *self
    }
}
