//! This crate provides common ACL facilities, namely the common groups and traits.
#[macro_use]
extern crate failure;
extern crate futures;

use futures::future;
use futures::prelude::*;

pub type Verdict<Context, E> = Box<Future<Item = (bool, Context), Error = (E, Context)>>;

#[derive(Clone, Debug, Fail)]
#[fail(display = "Unauthorized")]
pub struct UnauthorizedError;

/// Access control layer for repos. It tells if a user can do a certain action with
/// certain resource. All logic for roles and permissions should be hardcoded into implementation
/// of this trait.
pub trait AclEngine<Context, Error>
where
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    /// Tells if a user with id `user_id` can do `action` on `resource`.
    /// `resource_with_scope` can tell if this resource is in some scope, which is also a part of `acl` for some
    /// permissions. E.g. You can say that a user can do `Create` (`Action`) on `Store` (`Resource`) only if he's the
    /// `Owner` (`Scope`) of the store.
    fn allows(&self, ctx: Context) -> Verdict<Context, Error>;

    fn ensure_access(&self, ctx: Context) -> Box<Future<Item = Context, Error = (Error, Context)>> {
        Box::new(self.allows(ctx).and_then(|(allowed, ctx)| {
            future::result(if allowed {
                Ok(ctx)
            } else {
                Err((Error::from(UnauthorizedError), ctx))
            })
        }))
    }
}

pub struct AsyncACLFn<F>(pub F);
pub struct SyncACLFn<F>(pub F);
pub struct InfallibleSyncACLFn<F>(pub F);

impl<F, Context, Error> AclEngine<Context, Error> for AsyncACLFn<F>
where
    F: Fn(Context) -> Verdict<Context, Error>,
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    fn allows(&self, ctx: Context) -> Verdict<Context, Error> {
        (self.0)(ctx)
    }
}

impl<F, Context, Error> AclEngine<Context, Error> for SyncACLFn<F>
where
    F: Fn(&mut Context) -> Result<bool, Error>,
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    fn allows(&self, mut ctx: Context) -> Verdict<Context, Error> {
        Box::new(future::result(match (self.0)(&mut ctx) {
            Ok(allowed) => Ok((allowed, ctx)),
            Err(e) => Err((e, ctx)),
        }))
    }
}

impl<F, Context, Error> AclEngine<Context, Error> for InfallibleSyncACLFn<F>
where
    F: Fn(&mut Context) -> bool,
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    fn allows(&self, mut ctx: Context) -> Verdict<Context, Error> {
        let allowed = (self.0)(&mut ctx);
        Box::new(future::ok((allowed, ctx)))
    }
}

/// `SystemACL` allows all manipulation with resources in all cases.
#[derive(Clone, Debug, Default)]
pub struct SystemACL;

#[allow(unused)]
impl<Context, Error> AclEngine<Context, Error> for SystemACL
where
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    fn allows(&self, ctx: Context) -> Verdict<Context, Error> {
        Box::new(future::ok((true, ctx)))
    }
}

/// `ForbiddenACL` denies all manipulation with resources in all cases.
#[derive(Clone, Debug, Default)]
pub struct ForbiddenACL;

#[allow(unused)]
impl<Context, Error> AclEngine<Context, Error> for ForbiddenACL
where
    Context: 'static,
    Error: From<UnauthorizedError> + 'static,
{
    fn allows(&self, ctx: Context) -> Verdict<Context, Error> {
        Box::new(future::ok((false, ctx)))
    }
}
