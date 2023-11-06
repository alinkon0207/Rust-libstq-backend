use models::*;
use service::*;

use futures::prelude::*;
use hyper::{Body, Delete, Get, Method, Post};
use serde::{de::DeserializeOwned, Serialize};
use std::rc::Rc;
use stq_http::controller::ControllerFuture;
use stq_http::request_util::*;
use stq_router::Builder as RouterBuilder;
use stq_types::*;

#[derive(Clone, Debug)]
pub enum Route {
    Roles,
    RoleById(RoleEntryId),
    RolesByUserId(UserId),
}

pub fn add_routes<R>(b: RouterBuilder<R>) -> RouterBuilder<R>
where
    R: From<Route>,
{
    b.with_route(r"^/roles$", |_| Some(Route::Roles.into()))
        .with_route(r"^/roles/by-user-id/(\d+)$", |params| {
            params
                .get(0)
                .and_then(|string_id| string_id.parse().ok())
                .map(|v| Route::RolesByUserId(v).into())
        })
        .with_route(r"^/roles/by-id/([a-zA-Z0-9-]+)$", |params| {
            params
                .get(0)
                .and_then(|string_id| string_id.parse().ok())
                .map(|v| Route::RoleById(v).into())
        })
}

pub struct Controller<T> {
    pub service: Rc<RoleService<T>>,
}

impl<T> Controller<T>
where
    T: Serialize + DeserializeOwned + 'static,
{
    pub fn call(&self, method: &Method, route: &Route, payload: Body) -> Option<ControllerFuture> {
        let service = self.service.clone();

        match (method, route) {
            (Get, Route::RolesByUserId(user_id)) => Some({
                let user_id = *user_id;
                serialize_future({ service.get_roles_for_user(user_id) })
            }),
            (Post, Route::Roles) => Some(serialize_future({
                parse_body::<RoleEntry<T>>(payload).and_then(move |data| service.create_role(data))
            })),
            (Delete, Route::RolesByUserId(user_id)) => Some({
                let user_id = *user_id;
                serialize_future({
                    parse_body::<Option<T>>(payload).and_then(move |role| service.remove_role(RoleSearchTerms::Meta((user_id, role))))
                })
            }),
            (Delete, Route::RoleById(role_id)) => Some({
                let role_id = *role_id;
                serialize_future({ service.remove_role(RoleSearchTerms::Id(role_id)) })
            }),
            (_, _) => None,
        }
    }
}
