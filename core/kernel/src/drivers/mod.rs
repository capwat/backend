pub mod data;
pub mod storage;

pub use data::Data;
pub use storage::Storage;

// Common types for implementing ACTUAL microservices in Capwat
pub mod prelude {
  pub use crate::error::ext::*;
  pub use crate::error::{Error as KError, Result as KResult};

  #[cfg(feature = "grpc")]
  pub use crate::grpc::proto::{self, FromProto, FromRefProto, ToProto};

  pub use capwat_types::Sensitive;

  #[cfg(feature = "grpc")]
  pub use tonic::{
    Request as GrpcRequest, Response as GrpcResponse, Result as GrpcResult,
    Status as GrpcError,
  };
}
