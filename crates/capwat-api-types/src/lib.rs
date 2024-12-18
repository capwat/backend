pub mod error;
pub mod routes;
pub mod user;
pub mod util;

pub use self::error::{Error, ErrorCategory};

pub mod post {
    use serde::{Deserialize, Serialize};

    use crate::user::UserView;
    use crate::util::Timestamp;

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
    pub struct Post {
        pub id: i64,
        pub created_at: Timestamp,
        pub last_edited_at: Option<Timestamp>,
        pub author: UserView,
        pub content: String,
    }
}
