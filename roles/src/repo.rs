use models::*;

use stq_acl::*;
use stq_db::repo::*;
use stq_db::statement::{UpdateBuilder, Updater};

const TABLE: &str = "roles";

pub struct DummyRoleUpdater;

impl Updater for DummyRoleUpdater {
    fn into_update_builder(self, _table: &'static str) -> UpdateBuilder {
        unreachable!()
    }
}

pub trait RolesRepo<T>: DbRepo<RoleEntry<T>, RoleEntry<T>, RoleFilter<T>, DummyRoleUpdater, RepoError>
where
    T: RoleModel,
{
}

pub type RolesRepoImpl<T> = DbRepoImpl<RoleEntry<T>, RoleEntry<T>, RoleFilter<T>, DummyRoleUpdater>;
impl<T> RolesRepo<T> for RolesRepoImpl<T> where T: RoleModel {}

pub fn make_su_repo<T>() -> RolesRepoImpl<T>
where
    T: RoleModel,
{
    RolesRepoImpl::new(TABLE)
}

type AclContext<T> = (RoleEntry<T>, Action);

fn check_acl<T>(login: RepoLogin<T>, (entry, action): &mut AclContext<T>) -> bool
where
    T: RoleModel,
{
    use self::RepoLogin::*;

    match login {
        Anonymous => false,
        User { caller_id, caller_roles } => {
            for user_role in caller_roles {
                // Superadmins can do anything.
                if user_role.role.is_su() {
                    return true;
                }
            }

            // Others can only view their roles.
            if *action == Action::Select && caller_id == entry.user_id {
                return true;
            }

            false
        }
    }
}

/// Creates roles repo. No access for anonymous users, sorry.
pub fn make_repo<T>(login: RepoLogin<T>) -> RolesRepoImpl<T>
where
    T: RoleModel,
{
    make_su_repo().with_afterop_acl_engine(InfallibleSyncACLFn(move |ctx: &mut AclContext<T>| check_acl(login.clone(), ctx)))
}
