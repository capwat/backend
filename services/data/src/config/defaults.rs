use std::num::{NonZeroU32, NonZeroU64};

const DEFAULT_POOL_SIZE: u32 = 5;
const DEFAULT_POOL_TIMEOUT_SECS: u64 = 5;

pub const fn default_enforce_tls() -> bool {
  true
}

pub const fn default_pool_size() -> NonZeroU32 {
  match NonZeroU32::new(DEFAULT_POOL_SIZE) {
    Some(n) => n,
    None => panic!("DEFAULT_POOL_SIZE is accidentally set to 0"),
  }
}

pub const fn default_pool_timeout_secs() -> NonZeroU64 {
  match NonZeroU64::new(DEFAULT_POOL_TIMEOUT_SECS) {
    Some(n) => n,
    None => panic!("DEFAULT_POOL_TIMEOUT_SECS is accidentally set to 0"),
  }
}
