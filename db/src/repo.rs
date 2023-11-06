use super::connection::*;
use super::statement::{Filter, FilteredOperation, Inserter, SelectOperation, Updater};

use failure;
use futures::*;
use futures_state_stream::*;
use std::rc::Rc;
use stq_acl as acl;
use tokio_postgres::rows::Row;
use tokio_postgres::types::ToSql;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum MultipleOperationError {
    #[fail(display = "Operation has returned no data")]
    NoData,
    #[fail(display = "Operation returned extra data: +{}", extra)]
    ExtraData { extra: u32 },
}

pub trait DbRepoInsert<T: 'static, I: Inserter, E: From<MultipleOperationError> + 'static> {
    fn insert(&self, conn: BoxedConnection<E>, inserter: I) -> ConnectionFuture<Vec<T>, E>;

    fn insert_exactly_one(&self, conn: BoxedConnection<E>, inserter: I) -> ConnectionFuture<T, E> {
        Box::new(self.insert(conn, inserter).and_then(|(mut data, conn)| {
            if data.len() > 1 {
                Err((
                    E::from(MultipleOperationError::ExtraData {
                        extra: data.len() as u32 - 1,
                    }),
                    conn,
                ))
            } else if data.is_empty() {
                Err((E::from(MultipleOperationError::NoData), conn))
            } else if data.len() == 1 {
                Ok((data.pop().unwrap(), conn))
            } else {
                unreachable!()
            }
        }))
    }
}

pub trait DbRepoSelect<T: 'static, F: Filter, E: From<MultipleOperationError> + 'static> {
    fn select_full(
        &self,
        conn: BoxedConnection<E>,
        filter: F,
        limit: Option<i32>,
        op: Option<SelectOperation>,
    ) -> ConnectionFuture<Vec<T>, E>;

    fn select(&self, conn: BoxedConnection<E>, filter: F) -> ConnectionFuture<Vec<T>, E> {
        self.select_full(conn, filter, None, None)
    }

    fn select_exactly_one(&self, conn: BoxedConnection<E>, filter: F) -> ConnectionFuture<T, E> {
        Box::new(self.select(conn, filter).and_then(|(mut data, conn)| {
            if data.len() > 1 {
                Err((
                    E::from(MultipleOperationError::ExtraData {
                        extra: data.len() as u32 - 1,
                    }),
                    conn,
                ))
            } else if data.is_empty() {
                Err((E::from(MultipleOperationError::NoData), conn))
            } else if data.len() == 1 {
                Ok((data.pop().unwrap(), conn))
            } else {
                unreachable!()
            }
        }))
    }
}

pub trait DbRepoUpdate<T: 'static, U: Updater, E: From<MultipleOperationError> + 'static> {
    fn update(&self, conn: BoxedConnection<E>, updater: U) -> ConnectionFuture<Vec<T>, E>;

    fn update_exactly_one(&self, conn: BoxedConnection<E>, updater: U) -> ConnectionFuture<T, E> {
        Box::new(self.update(conn, updater).and_then(|(mut data, conn)| {
            if data.len() > 1 {
                Err((
                    E::from(MultipleOperationError::ExtraData {
                        extra: data.len() as u32 - 1,
                    }),
                    conn,
                ))
            } else if data.is_empty() {
                Err((E::from(MultipleOperationError::NoData), conn))
            } else if data.len() == 1 {
                Ok((data.pop().unwrap(), conn))
            } else {
                unreachable!()
            }
        }))
    }
}

pub trait DbRepoDelete<T: 'static, F: Filter, E: From<MultipleOperationError> + 'static> {
    fn delete(&self, conn: BoxedConnection<E>, filter: F) -> ConnectionFuture<Vec<T>, E>;

    fn delete_exactly_one(&self, conn: BoxedConnection<E>, filter: F) -> ConnectionFuture<T, E> {
        Box::new(self.delete(conn, filter).and_then(|(mut data, conn)| {
            if data.len() > 1 {
                Err((
                    E::from(MultipleOperationError::ExtraData {
                        extra: data.len() as u32 - 1,
                    }),
                    conn,
                ))
            } else if data.is_empty() {
                Err((E::from(MultipleOperationError::NoData), conn))
            } else if data.len() == 1 {
                Ok((data.pop().unwrap(), conn))
            } else {
                unreachable!()
            }
        }))
    }
}

pub trait DbRepo<T: 'static, I: Inserter, F: Filter, U: Updater, E: From<MultipleOperationError> + 'static>:
    DbRepoInsert<T, I, E> + DbRepoSelect<T, F, E> + DbRepoDelete<T, F, E> + DbRepoUpdate<T, U, E>
{
}

pub type RepoError = failure::Error;
pub type RepoFuture<T> = Box<Future<Item = T, Error = RepoError>>;
pub type RepoConnection = BoxedConnection<RepoError>;
pub type RepoConnectionFuture<T> = ConnectionFuture<T, RepoError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Insert,
    Select,
    Delete,
    Update,
}

fn bulk_ensure_access<T>(
    acl_engine: &Rc<acl::AclEngine<(T, Action), RepoError>>,
    context: (Vec<T>, Action),
    conn: BoxedConnection<RepoError>,
) -> impl Future<Item = (Vec<T>, BoxedConnection<RepoError>), Error = (RepoError, BoxedConnection<RepoError>)>
where
    T: 'static,
{
    let (items, action) = context;
    future::join_all(items.into_iter().map({
        let acl_engine = acl_engine.clone();
        move |entity| acl_engine.ensure_access((entity, action)).map(|(entity, _)| entity)
    }))
    .then(move |res| match res {
        Ok(items) => Ok((items, conn)),
        Err((e, _ctx)) => Err((e, conn)),
    })
}

pub struct DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    I: Inserter + 'static,
    F: Filter + 'static,
    U: Updater + 'static,
{
    pub table: &'static str,
    pub insert_acl_engine: Rc<acl::AclEngine<I, RepoError>>,
    pub select_acl_engine: Rc<acl::AclEngine<F, RepoError>>,
    pub delete_acl_engine: Rc<acl::AclEngine<F, RepoError>>,
    pub update_acl_engine: Rc<acl::AclEngine<U, RepoError>>,
    pub afterop_acl_engine: Rc<acl::AclEngine<(T, Action), RepoError>>,
}

impl<T, I, F, U> DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    F: Filter + 'static,
    I: Inserter + 'static,
    U: Updater + 'static,
{
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            insert_acl_engine: Rc::new(acl::SystemACL),
            select_acl_engine: Rc::new(acl::SystemACL),
            delete_acl_engine: Rc::new(acl::SystemACL),
            update_acl_engine: Rc::new(acl::SystemACL),
            afterop_acl_engine: Rc::new(acl::SystemACL),
        }
    }

    pub fn with_insert_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: acl::AclEngine<I, RepoError> + 'static,
    {
        self.insert_acl_engine = Rc::new(acl_engine);
        self
    }

    pub fn with_select_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: acl::AclEngine<F, RepoError> + 'static,
    {
        self.select_acl_engine = Rc::new(acl_engine);
        self
    }

    pub fn with_delete_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: acl::AclEngine<F, RepoError> + 'static,
    {
        self.delete_acl_engine = Rc::new(acl_engine);
        self
    }

    pub fn with_update_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: acl::AclEngine<U, RepoError> + 'static,
    {
        self.update_acl_engine = Rc::new(acl_engine);
        self
    }

    pub fn with_afterop_acl_engine<E>(mut self, acl_engine: E) -> Self
    where
        E: acl::AclEngine<(T, Action), RepoError> + 'static,
    {
        self.afterop_acl_engine = Rc::new(acl_engine);
        self
    }
}

fn query_debug(q: &str, args: &[Box<ToSql>]) -> String {
    let args_dbg = args.iter().enumerate().fold(String::new(), |mut acc, (i, arg)| {
        if i > 0 {
            acc += ", ";
        }
        acc += &format!("${} = {:?}", i + 1, arg);
        acc
    });

    format!("Query: {}. Args: {}", q, &args_dbg)
}

impl<T, I, F, U> DbRepoInsert<T, I, RepoError> for DbRepoImpl<T, I, F, U>
where
    F: Filter,
    T: From<Row> + 'static,
    I: Inserter,
    U: Updater,
{
    fn insert(&self, conn: RepoConnection, inserter: I) -> RepoConnectionFuture<Vec<T>> {
        let table = self.table;

        let afterop_acl_engine = self.afterop_acl_engine.clone();

        Box::new(
            self.insert_acl_engine
                .ensure_access(inserter)
                .then(move |res| {
                    future::result(match res {
                        Ok(inserter) => {
                            let (query, args) = inserter.into_insert_builder(table).build();
                            Ok((query, args, conn))
                        }
                        Err((e, _inserter)) => Err((e, conn)),
                    })
                })
                .and_then(move |(query, args, conn)| conn.prepare2(&query).map(move |(statement, conn)| (statement, query, args, conn)))
                .and_then(move |(statement, query, args, conn)| {
                    let err_msg = query_debug(&query, &args);
                    conn.query2(&statement, args)
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                })
                .map(|(rows, conn)| (rows.into_iter().map(T::from).collect::<Vec<T>>(), conn))
                .and_then(move |(items, conn)| bulk_ensure_access(&afterop_acl_engine, (items, Action::Insert), conn))
                .map_err(|(e, conn)| (e.context("Failure while running insert").into(), conn)),
        )
    }
}

impl<T, I, F, U> DbRepoSelect<T, F, RepoError> for DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    F: Filter,
    I: Inserter,
    U: Updater,
{
    fn select_full(
        &self,
        conn: RepoConnection,
        filter: F,
        limit: Option<i32>,
        op: Option<SelectOperation>,
    ) -> RepoConnectionFuture<Vec<T>> {
        let table = self.table;

        let afterop_acl_engine = self.afterop_acl_engine.clone();

        Box::new(
            self.select_acl_engine
                .ensure_access(filter)
                .then(move |res| match res {
                    Ok(filter) => {
                        if let Some(limit) = limit {
                            if limit < 1 {
                                return Box::new(future::err((format_err!("Limit cannot be less than 1"), conn)));
                            }
                        }

                        let (query, args) = filter
                            .into_filtered_operation_builder(table)
                            .build(FilteredOperation::Select { op, limit });
                        Box::new(future::ok((query, args, conn)))
                    }
                    Err((e, _filter)) => Box::new(future::err((e, conn))),
                })
                .and_then(move |(query, args, conn)| conn.prepare2(&query).map(move |(statement, conn)| (statement, query, args, conn)))
                .and_then(move |(statement, query, args, conn)| {
                    let err_msg = query_debug(&query, &args);
                    conn.query2(&statement, args)
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                })
                .map(|(rows, conn)| (rows.into_iter().map(T::from).collect::<Vec<T>>(), conn))
                .and_then(move |(items, conn)| bulk_ensure_access(&afterop_acl_engine, (items, Action::Select), conn))
                .map_err(|(e, conn)| (e.context("Failure while running select").into(), conn)),
        )
    }
}

impl<T, I, F, U> DbRepoUpdate<T, U, RepoError> for DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    F: Filter,
    I: Inserter,
    U: Updater,
{
    fn update(&self, conn: RepoConnection, updater: U) -> RepoConnectionFuture<Vec<T>> {
        let table = self.table;

        let afterop_acl_engine = self.afterop_acl_engine.clone();

        Box::new(
            self.update_acl_engine
                .ensure_access(updater)
                .then(move |res| {
                    future::result(match res {
                        Ok(updater) => {
                            let (query, args) = updater.into_update_builder(table).build();
                            Ok((query, args, conn))
                        }
                        Err((e, _updater)) => Err((e, conn)),
                    })
                })
                .and_then(move |(query, args, conn)| conn.prepare2(&query).map(move |(statement, conn)| (statement, query, args, conn)))
                .and_then(move |(statement, query, args, conn)| {
                    let err_msg = query_debug(&query, &args);
                    conn.query2(&statement, args)
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                })
                .map(|(rows, conn)| (rows.into_iter().map(T::from).collect::<Vec<T>>(), conn))
                .and_then(move |(items, conn)| bulk_ensure_access(&afterop_acl_engine, (items, Action::Update), conn))
                .map_err(|(e, conn)| (e.context("Failure while running update").into(), conn)),
        )
    }
}

impl<T, I, F, U> DbRepoDelete<T, F, RepoError> for DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    F: Filter,
    I: Inserter,
    U: Updater,
{
    fn delete(&self, conn: RepoConnection, filter: F) -> RepoConnectionFuture<Vec<T>> {
        let table = self.table;

        let afterop_acl_engine = self.afterop_acl_engine.clone();

        Box::new(
            self.delete_acl_engine
                .ensure_access(filter)
                .then(move |res| {
                    future::result(match res {
                        Ok(filter) => {
                            let (query, args) = filter.into_filtered_operation_builder(table).build(FilteredOperation::Delete);
                            Ok((query, args, conn))
                        }
                        Err((e, _filter)) => Err((e, conn)),
                    })
                })
                .and_then(move |(query, args, conn)| conn.prepare2(&query).map(move |(statement, conn)| (statement, query, args, conn)))
                .and_then(move |(statement, query, args, conn)| {
                    let err_msg = query_debug(&query, &args);
                    conn.query2(&statement, args)
                        .collect()
                        .map_err(move |(e, conn)| (e.context(err_msg).into(), conn))
                })
                .map(|(rows, conn)| (rows.into_iter().map(T::from).collect::<Vec<T>>(), conn))
                .and_then(move |(items, conn)| bulk_ensure_access(&afterop_acl_engine, (items, Action::Delete), conn))
                .map_err(|(e, conn)| (e.context("Failure while running delete").into(), conn)),
        )
    }
}

impl<T, I, F, U> DbRepo<T, I, F, U, RepoError> for DbRepoImpl<T, I, F, U>
where
    T: From<Row> + 'static,
    F: Filter,
    I: Inserter,
    U: Updater,
{
}
