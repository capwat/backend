mod base64;
pub mod timestamp;

pub use self::base64::EncodedBase64;
pub use self::timestamp::Timestamp;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum SortOrder {
    #[default]
    #[serde(rename = "desc")]
    Descending,
    #[serde(rename = "asc")]
    Ascending,
}

crate::should_impl_primitive_traits!(SortOrder);
