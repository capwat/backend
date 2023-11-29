use capwat_kernel::services::DataService;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct App {
  pub data: Arc<dyn DataService>,
}
