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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PermissionLevel {
    User,
    Moderator,
    Superuser,
}

pub struct CommandResponse {
    pub response: String,

    // Can the output contain a banphrase?
    // i.e. can the user manipulate the response in a non-controlled way
    pub questionable_output: bool,
}

impl CommandResponse {
    pub fn new(response: String, questionable_output: bool) -> Self {
        Self {
            response,
            questionable_output,
        }
    }
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PermissionLevel::Superuser => write!(f, "superuser"),
            PermissionLevel::Moderator => write!(f, "moderator"),
            PermissionLevel::User => write!(f, "user"),
        }
    }
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

    // checks if the permission level 'self' is equal too or supersedes 'permission_needed'
    // essentially a greater/less than or equal to comparison for PermissionLevel enums
    pub fn satisfies(self, permission_needed: PermissionLevel) -> bool {
        match self {
            PermissionLevel::Superuser => true,
            PermissionLevel::Moderator => {
                (permission_needed == PermissionLevel::Moderator)
                    || (permission_needed == PermissionLevel::User)
            }
            PermissionLevel::User => permission_needed == PermissionLevel::User,
        }
    }
}
