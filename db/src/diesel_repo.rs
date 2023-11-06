use diesel::{connection::AnsiTransactionManager, pg::Pg, query_dsl::LoadQuery, Connection};
use failure::{self, Fallible};
use std::sync::Arc;
use stq_acl::*;

use super::repo::Action;

/// Repository responsible for handling products
pub struct DieselRepoImpl<'a, Conn, Output>
where
    Conn: 'a,
{
    pub db_conn: &'a Conn,
    pub acl_engine: Arc<AclEngine<(Output, Action), failure::Error>>,
}

impl<'a, Conn, Output> DieselRepoImpl<'a, Conn, Output>
where
    Conn: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    Output: Send + 'static,
{
    pub fn new(db_conn: &'a Conn) -> Self {
        Self {
            db_conn,
            acl_engine: Arc::new(SystemACL),
        }
    }

    pub fn with_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: AclEngine<(Output, Action), failure::Error> + 'static,
    {
        self.acl_engine = Arc::new(acl_engine);
        self
    }

    pub fn execute_query<U>(&self, query: U) -> Fallible<Output>
    where
        U: LoadQuery<Conn, Output> + Send + 'static,
    {
        query.get_result::<Output>(self.db_conn).map_err(From::from)
    }
}
