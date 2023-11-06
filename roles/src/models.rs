use failure;
use futures::prelude::*;
use serde_json::Value;
use std::fmt::Debug;
use std::rc::Rc;
use stq_db::statement::*;
use stq_types::*;
use tokio_postgres::rows::Row;

pub const ID_COLUMN: &str = "id";
pub const USER_ID_COLUMN: &str = "user_id";
pub const ROLE_NAME_COLUMN: &str = "name";
pub const ROLE_DATA_COLUMN: &str = "data";

pub trait RoleModel: Clone + Debug + 'static {
    fn is_su(&self) -> bool;
    fn from_db(variant: &str, data: Value) -> Result<Self, failure::Error>;
    fn into_db(self) -> (String, Value);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoleEntry<T> {
    pub id: RoleEntryId,
    pub user_id: UserId,
    pub role: T,
}

impl<T> From<Row> for RoleEntry<T>
where
    T: RoleModel,
{
    fn from(row: Row) -> Self {
        Self {
            id: RoleEntryId(row.get(ID_COLUMN)),
            user_id: UserId(row.get(USER_ID_COLUMN)),
            role: T::from_db(row.get(ROLE_NAME_COLUMN), row.get(ROLE_DATA_COLUMN)).unwrap(),
        }
    }
}

impl<T> Inserter for RoleEntry<T>
where
    T: RoleModel,
{
    fn into_insert_builder(self, table: &'static str) -> InsertBuilder {
        let (role_name, role_data) = T::into_db(self.role);
        InsertBuilder::new(table)
            .with_arg(ID_COLUMN, self.id.0)
            .with_arg(USER_ID_COLUMN, self.user_id.0)
            .with_arg(ROLE_NAME_COLUMN, role_name)
            .with_arg(ROLE_DATA_COLUMN, role_data)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoleFilter<T> {
    pub id: Option<RoleEntryId>,
    pub user_id: Option<UserId>,
    pub role: Option<T>,
}

impl<T> Default for RoleFilter<T> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            user_id: Default::default(),
            role: Default::default(),
        }
    }
}

impl<T> Filter for RoleFilter<T>
where
    T: RoleModel,
{
    fn into_filtered_operation_builder(self, table: &'static str) -> FilteredOperationBuilder {
        let mut b = FilteredOperationBuilder::new(table);

        if let Some(id) = self.id {
            b = b.with_filter(ID_COLUMN, id.0);
        }

        if let Some(user_id) = self.user_id {
            b = b.with_filter(USER_ID_COLUMN, user_id.0);
        }

        if let Some(role) = self.role {
            let (role_name, role_data) = T::into_db(role);
            b = b.with_filter(ROLE_NAME_COLUMN, role_name).with_filter(ROLE_DATA_COLUMN, role_data);
        }

        b
    }
}

#[derive(Clone, Debug)]
pub enum RoleSearchTerms<T> {
    Id(RoleEntryId),
    Meta((UserId, Option<T>)),
}

impl<T> From<RoleSearchTerms<T>> for RoleFilter<T> {
    fn from(v: RoleSearchTerms<T>) -> Self {
        use self::RoleSearchTerms::*;

        match v {
            Id(id) => Self {
                id: Some(id),
                ..Default::default()
            },
            Meta((user_id, role)) => Self {
                user_id: Some(user_id),
                role,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum RepoLogin<T> {
    Anonymous,
    User {
        caller_id: UserId,
        caller_roles: Vec<RoleEntry<T>>,
    },
}

pub type ServiceFuture<T> = Box<Future<Item = T, Error = failure::Error>>;
pub type RepoLoginFuture<T> = ServiceFuture<RepoLogin<T>>;
pub type RepoLoginSource<T> = Rc<Fn() -> RepoLoginFuture<T>>;
