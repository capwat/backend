pub mod category;
pub use self::category::ErrorCategory;

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
