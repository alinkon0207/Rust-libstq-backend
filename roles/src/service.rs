use models::*;
use repo::*;

use futures::future;
use futures::prelude::*;
use std::fmt::Debug;
use std::rc::Rc;
use stq_db::pool::Pool as DbPool;
use stq_db::repo::*;
use stq_types::*;

pub fn get_login_data<T>(db_pool: &DbPool, caller_id: Option<UserId>) -> RepoLoginFuture<T>
where
    T: RoleModel,
{
    match caller_id {
        None => Box::new(future::ok(RepoLogin::Anonymous)),
        Some(caller_id) => Box::new(
            db_pool
                .run(move |conn| make_su_repo().select(conn, RoleSearchTerms::Meta((caller_id, None)).into()))
                .map_err(|e| e.context("Failed to fetch user roles").into())
                .map(move |caller_roles| RepoLogin::User { caller_id, caller_roles }),
        ),
    }
}

pub trait RoleService<T> {
    fn get_roles_for_user(&self, user_id: UserId) -> ServiceFuture<Vec<RoleEntry<T>>>;
    fn create_role(&self, item: RoleEntry<T>) -> ServiceFuture<RoleEntry<T>>;
    fn remove_role(&self, filter: RoleSearchTerms<T>) -> ServiceFuture<Option<RoleEntry<T>>>;
    fn remove_all_roles(&self, user_id: UserId) -> ServiceFuture<Vec<RoleEntry<T>>>;
}

pub struct RoleServiceImpl<T> {
    pub repo_factory: Rc<Fn() -> Box<RolesRepo<T>>>,
    pub db_pool: DbPool,
}

impl<T> RoleServiceImpl<T>
where
    T: RoleModel + Clone,
{
    pub fn new(db_pool: DbPool, login: RepoLogin<T>) -> Self {
        Self {
            db_pool,
            repo_factory: Rc::new(move || Box::new(make_repo(login.clone()))),
        }
    }
}

impl<T> RoleService<T> for RoleServiceImpl<T>
where
    T: RoleModel + Clone + Debug,
{
    fn get_roles_for_user(&self, user_id: UserId) -> ServiceFuture<Vec<RoleEntry<T>>> {
        let repo_factory = self.repo_factory.clone();
        let db_pool = self.db_pool.clone();
        Box::new(
            db_pool
                .run(move |conn| (repo_factory)().select(conn, RoleSearchTerms::Meta((user_id, None)).into()))
                .map_err(move |e| e.context(format!("Failed to get roles for user {}", user_id.0)).into()),
        )
    }
    fn create_role(&self, item: RoleEntry<T>) -> ServiceFuture<RoleEntry<T>> {
        let repo_factory = self.repo_factory.clone();
        let db_pool = self.db_pool.clone();
        Box::new(
            db_pool
                .run({
                    let item = item.clone();
                    move |conn| (repo_factory)().insert_exactly_one(conn, item)
                })
                .map_err(move |e| e.context(format!("Failed to create role: {:?}", item)).into()),
        )
    }
    fn remove_role(&self, filter: RoleSearchTerms<T>) -> ServiceFuture<Option<RoleEntry<T>>> {
        let repo_factory = self.repo_factory.clone();
        let db_pool = self.db_pool.clone();
        Box::new(
            db_pool
                .run({
                    let filter = filter.clone();
                    move |conn| (repo_factory)().delete(conn, filter.into())
                })
                .map(|mut v| v.pop())
                .map_err(move |e| e.context(format!("Failed to remove role: {:?}", filter)).into()),
        )
    }
    fn remove_all_roles(&self, user_id: UserId) -> ServiceFuture<Vec<RoleEntry<T>>> {
        let repo_factory = self.repo_factory.clone();
        let db_pool = self.db_pool.clone();
        Box::new(
            db_pool
                .run(move |conn| (repo_factory)().delete(conn, RoleSearchTerms::Meta((user_id, None)).into()))
                .map_err(move |e| e.context(format!("Failed to remove all roles for user {}", user_id.0)).into()),
        )
    }
}
