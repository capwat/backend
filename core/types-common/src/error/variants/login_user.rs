use capwat_types_derive::CategoryError;
use either::Either;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{error::SubcategoryMessage, Timestamp};

#[derive(Debug, Clone, CategoryError, PartialEq, Eq)]
pub enum LoginUser {
    #[error(subcode = 1)]
    #[error(message = "Invalid credientials!")]
    InvalidCredientials,
    #[error(subcode = 2)]
    Banned(Box<LoginUserBanData>),
}

impl Display for LoginUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("User tried to login ")?;
        match self {
            LoginUser::InvalidCredientials => {
                f.write_str("with invalid credentials")
            },
            LoginUser::Banned(..) => f.write_str("with their banned account"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoginUserBanData {
    pub appealable: bool,
    pub banned_until: Option<Timestamp>,
    pub reason: String,
    pub violations: Vec<String>,
}

impl SubcategoryMessage for LoginUserBanData {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Your account has been terminated")
    }
}
