#[derive(Debug)]
pub struct UserContext {
    pub uid: i32,
    pub login: String,
    pub permissions: PermissionLevel,
}

impl UserContext {
    pub fn new(uid: i32, login: String, permission_num: i32) -> Self {
        UserContext {
            uid,
            login,
            permissions: PermissionLevel::new(permission_num),
        }
    }
}

#[derive(Debug)]
pub enum PermissionLevel {
    User,
    Moderator,
    Superuser,
}

impl PermissionLevel {
    pub fn new(permission_num: i32) -> PermissionLevel {
        match permission_num {
            0 => PermissionLevel::User,
            1 => PermissionLevel::Moderator,
            2 => PermissionLevel::Superuser,
            _ => PermissionLevel::User,
        }
    }
}
