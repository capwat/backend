use super::{
    Category, CategoryMessage, DeserializeCategory, SerializeCategory,
};
use crate::internal::Sealed;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unknown {
    pub code: u64,
    pub subcode: Option<u64>,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl Sealed for Unknown {}

impl DeserializeCategory for Unknown {
    // This is to force to create `Unknown` object automatically
    // directly into the derive expanded code.
    fn deserialize<D: serde::de::Error>(
        _subcode: Option<u64>,
        _data: Option<serde_json::Value>,
    ) -> Result<either::Either<Self, Option<serde_json::Value>>, D> {
        Ok(either::Either::Right(None))
    }
}

impl SerializeCategory for Unknown {
    fn has_data(&self) -> bool {
        self.data.is_some()
    }

    fn serialize_data<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        self.data.serialize(serializer)
    }
}

impl CategoryMessage for Unknown {
    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Display;
        Display::fmt(&self.message, f)
    }
}

impl Category for Unknown {
    fn has_subcode(&self) -> bool {
        self.subcode.is_some()
    }

    fn subcode(&self) -> Option<u64> {
        self.subcode
    }
}
