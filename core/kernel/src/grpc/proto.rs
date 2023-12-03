mod data {
  tonic::include_proto!("data");
}
mod schema {
  tonic::include_proto!("schema");
}

pub use data::*;
pub use schema::*;

pub trait FromProto {
  type ProtoType: prost::Message;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized;
}

pub trait FromRefProto<'a> {
  type ProtoType: prost::Message;

  fn from_ref_proto(proto: &'a Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized + 'a;
}

pub trait ToProto {
  type ProtoType: prost::Message;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized;
}
