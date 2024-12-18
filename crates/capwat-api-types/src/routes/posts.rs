use serde::{Deserialize, Serialize};

/// Get a list of posts recommended by Capwat for the current user.
///
/// The algorithm to get the recommended posts for the user will
/// depend on the instance administrators how they configure it.
///
/// By default, it chooses posts from whom got followed by the current
/// user and sorts it by its creation time to avoid exploiting/over-optimizing
/// the Capwat algorithm for content creators but again, it depends on how the
/// instance administrators will configure it and it must be kept
/// secret to everyone.
///
/// This object must be used as query parameters to perform
/// this request.
///
/// **ROUTE**: `GET /posts/recommendations`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListPostRecommendations {
    /// The highest post ID in the previous page.
    pub after: Option<i64>,
    /// Maximum number of posts to fetch (1-15)
    pub limit: Option<u64>,
}
