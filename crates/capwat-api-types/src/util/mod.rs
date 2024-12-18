mod base64;
pub mod timestamp;

pub use self::base64::EncodedBase64;
pub use self::timestamp::Timestamp;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Pagination {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
