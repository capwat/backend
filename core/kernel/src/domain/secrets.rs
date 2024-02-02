use async_trait::async_trait;
use capwat_types::Sensitive;

use crate::Result;

/// A secret manager allows to retrieve secrets carefully depending
/// on the object that is passed on with this trait implemented.
///
/// **⚠️ If you're willing to implement this trait, it is your responsibility
/// to handle secrets securely and safely and your implementation of this trait
/// on the object you're going to pass on to avoid any possible security
/// and data breaches.**
#[async_trait]
pub trait Manager: Send + Sync {
    /// Gets the JWT secret key, used to verify users' JWT tokens.
    async fn jwt_key(&self) -> Result<Sensitive<String>>;
}
