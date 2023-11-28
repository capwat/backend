use super::ErrorCategory;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub struct Ignored(());

impl Display for Ignored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Ignored error message")
  }
}

impl ErrorCategory for Ignored {
  fn code() -> u32 {
    u32::MAX
  }

  fn subcode(&self) -> Option<u32> {
    None
  }

  #[cfg(feature = "server_impl")]
  fn server_message(
    &self,
    _f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    Ok(())
  }

  fn message(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Ok(())
  }

  fn needs_data_serialization(&self) -> bool {
    false
  }

  fn deserialize_data<'de, D>(
    _subcode: Option<u32>,
    _deserializer: D,
  ) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
    Self: Sized,
  {
    Err(serde::de::Error::custom("Ignored should not be used"))
  }
}
