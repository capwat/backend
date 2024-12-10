use std::ops::Deref;

use crate::{user::UserKeys, User};

#[derive(Debug, Clone)]
pub struct UserView {
    pub user: User,
    pub current_keys: UserKeys,
}

impl Deref for UserView {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}
