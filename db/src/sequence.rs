use connection::*;

use failure;
use futures::future;
use futures::prelude::*;
use futures_state_stream::StateStream;
use std::fmt::Display;
use tokio_postgres::rows::Row;
use tokio_postgres::types::FromSql;

pub type SequenceError = failure::Error;
pub type SequenceFuture<T> = Box<Future<Item = T, Error = SequenceError>>;
pub type SequenceConnection = BoxedConnection<SequenceError>;
pub type SequenceConnectionFuture<T> = ConnectionFuture<T, SequenceError>;

pub unsafe trait Sequenceable: Clone + Display + FromSql<'static> + 'static {
    fn unmarshal_sequence_row(row: Row) -> Self;
}
unsafe impl Sequenceable for i32 {
    fn unmarshal_sequence_row(row: Row) -> Self {
        row.get(0)
    }
}
unsafe impl Sequenceable for i64 {
    fn unmarshal_sequence_row(row: Row) -> Self {
        row.get(0)
    }
}

pub trait Sequence<T: Sequenceable> {
    fn next_val(&self, conn: SequenceConnection) -> SequenceConnectionFuture<T>;
    fn reset(&self, conn: SequenceConnection, to: Option<T>) -> SequenceConnectionFuture<()>;
}

pub struct SequenceImpl {
    pub sequence: &'static str,
}

impl SequenceImpl {
    pub fn new(sequence: &'static str) -> Self {
        Self { sequence }
    }
}

impl<T> Sequence<T> for SequenceImpl
where
    T: Sequenceable,
{
    fn next_val(&self, conn: SequenceConnection) -> SequenceConnectionFuture<T> {
        let sequence = self.sequence;

        let err_msg = format!("Failed to increment sequence {}", sequence);

        Box::new(
            conn.prepare2(&format!("SELECT nextval(\'{}\');", sequence))
                .and_then(|(stmt, conn)| {
                    conn.query2(&stmt, vec![])
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                        .and_then(|(mut rows, conn)| {
                            future::result(match rows.pop() {
                                None => Err((format_err!("No rows returned"), conn)),
                                Some(row) => Ok((T::unmarshal_sequence_row(row), conn)),
                            })
                        })
                }),
        )
    }

    fn reset(&self, conn: SequenceConnection, to: Option<T>) -> SequenceConnectionFuture<()> {
        let sequence = self.sequence;

        let err_msg = format!("Failed to reset sequence {}", sequence);

        Box::new({
            let mut q = format!("ALTER SEQUENCE {} RESTART", sequence);
            if let Some(v) = to {
                q += &format!(" WITH {}", v);
            }
            q.push(';');

            conn.prepare2(&q)
                .and_then(|(stmt, conn)| {
                    conn.query2(&stmt, vec![])
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                })
                .map(|(_, conn)| ((), conn))
        })
    }
}
