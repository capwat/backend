pub mod category;
pub use self::category::ErrorCategory;

use self::category::ErrorCode;

#[cfg(feature = "axum")]
mod axum;

#[derive(Debug, Clone)]
#[must_use]
pub struct Error {
    pub category: ErrorCategory,
    pub message: Option<String>,
}

impl Error {
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            category,
            message: None,
        }
    }

    pub fn unknown() -> Self {
        Self {
            category: ErrorCategory::Unknown,
            message: None,
        }
    }

    pub fn message(self, message: impl Into<String>) -> Self {
        Self {
            category: self.category,
            message: Some(message.into()),
        }
    }

    pub fn code(&self) -> ErrorCode {
        match &self.category {
            ErrorCategory::Unknown => ErrorCode::Unknown(None),
            ErrorCategory::ReadonlyMode => ErrorCode::ReadonlyMode(None),
            ErrorCategory::InvalidRequest => ErrorCode::InvalidRequest(None),
            ErrorCategory::Outage => ErrorCode::Outage(None),
            ErrorCategory::InstanceClosed => ErrorCode::InstanceClosed(None),
            ErrorCategory::NoEmailAddress => ErrorCode::NoEmailAddress(None),
            ErrorCategory::EmailVerificationRequired => ErrorCode::EmailVerificationRequired(None),
            ErrorCategory::LoginUserFailed(login_user_failed) => match login_user_failed {
                category::LoginUserFailed::InvalidCredientials => ErrorCode::LoginUserFailed(Some(
                    category::LoginUserFailedSubcode::InvalidCredientials,
                )),
            },
            ErrorCategory::RegisterUserFailed(register_user_failed) => match register_user_failed {
                category::RegisterUserFailed::Closed => {
                    ErrorCode::RegisterUserFailed(Some(category::RegisterUserFailedSubcode::Closed))
                }
                category::RegisterUserFailed::InvalidPassword => ErrorCode::RegisterUserFailed(
                    Some(category::RegisterUserFailedSubcode::InvalidPassword),
                ),
                category::RegisterUserFailed::UnmatchedPassword => ErrorCode::RegisterUserFailed(
                    Some(category::RegisterUserFailedSubcode::UnmatchedPassword),
                ),
                category::RegisterUserFailed::UsernameTaken => ErrorCode::RegisterUserFailed(Some(
                    category::RegisterUserFailedSubcode::UsernameTaken,
                )),
                category::RegisterUserFailed::EmailTaken => ErrorCode::RegisterUserFailed(Some(
                    category::RegisterUserFailedSubcode::EmailTaken,
                )),
                category::RegisterUserFailed::EmailRequired => ErrorCode::RegisterUserFailed(Some(
                    category::RegisterUserFailedSubcode::EmailRequired,
                )),
            },
            ErrorCategory::Other(other_error) => other_error.code.clone(),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.category == other.category
    }
}

impl Eq for Error {}

impl std::hash::Hash for Error {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.category.hash(state);
    }
}
