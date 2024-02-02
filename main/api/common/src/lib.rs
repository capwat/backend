use capwat_kernel::domain::{secrets, users};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Context {
    pub secrets: Arc<dyn secrets::Manager>,
    pub users: Arc<dyn users::Service>,
}
