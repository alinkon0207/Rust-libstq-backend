use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(Uuid);

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum Role {
    Superuser,
    User,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct NewUserRole {
    pub user_id: i32,
    pub role: Role,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct UserRole {
    pub id: RoleId,
    pub user_id: i32,
    pub role: Role,
}
