use rpc_client::RestApiClient;
use types::*;
use util::*;

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use stq_roles::models::*;
use stq_roles::routing::*;
use stq_types::UserId;

impl RouteBuilder for Route {
    fn route(&self) -> String {
        match self {
            Route::Roles => "roles".into(),
            Route::RoleById(entry_id) => format!("roles/by-id/{}", entry_id),
            Route::RolesByUserId(user_id) => format!("roles/by-user-id/{}", user_id),
        }
    }
}

pub trait RolesClient<T>
where
    T: RoleModel + Clone + Debug + Serialize + DeserializeOwned + Send,
{
    fn get_roles_for_user(&self, user_id: UserId) -> ApiFuture<T>;
    fn create_role(&self, item: RoleEntry<T>) -> ApiFuture<RoleEntry<T>>;
    fn remove_role(&self, terms: RoleSearchTerms<T>) -> ApiFuture<Option<RoleEntry<T>>>;
}

impl<T> RolesClient<T> for RestApiClient
where
    T: RoleModel + Clone + Debug + Serialize + DeserializeOwned + Send,
{
    fn get_roles_for_user(&self, user_id: UserId) -> ApiFuture<T> {
        http_req(
            self.http_client
                .get(&self.build_route(&Route::RolesByUserId(user_id))),
        )
    }

    fn create_role(&self, item: RoleEntry<T>) -> ApiFuture<RoleEntry<T>> {
        http_req(
            self.http_client
                .post(&self.build_route(&Route::Roles))
                .body(JsonPayload(item)),
        )
    }

    fn remove_role(&self, terms: RoleSearchTerms<T>) -> ApiFuture<Option<RoleEntry<T>>> {
        http_req(match terms {
            RoleSearchTerms::Id(id) => self
                .http_client
                .delete(&self.build_route(&Route::RoleById(id))),
            RoleSearchTerms::Meta((user_id, entry)) => self
                .http_client
                .delete(&self.build_route(&Route::RolesByUserId(user_id)))
                .body(JsonPayload(entry)),
        })
    }
}
