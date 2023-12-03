use capwat_kernel::drivers;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct App {
  data: Arc<dyn drivers::Data>,
}

impl App {
  #[must_use]
  pub fn new(data: Arc<impl drivers::Data + 'static>) -> Self {
    Self { data }
  }
}

impl App {
  #[must_use]
  pub fn data(&self) -> &dyn drivers::Data {
    self.data.as_ref()
  }
}
