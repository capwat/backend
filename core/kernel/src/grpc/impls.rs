use crate::{drivers, entity::Secret, error::ext::ErrorExt};
use capwat_types::{id::marker::Marker, Id, Sensitive};
use chrono::NaiveDateTime;

use super::proto::{self, FromProto, FromRefProto, ToProto};
use crate::entity::User;

impl FromProto for User {
  type ProtoType = proto::User;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized,
  {
    Ok(Self {
      id: Id::from_proto(proto.id)?,
      created_at: NaiveDateTime::from_proto(proto.created_at)?,
      name: proto.name,
      email: proto.email,
      display_name: proto.display_name,
      password_hash: proto.password_hash,
      updated_at: proto
        .updated_at
        .map(NaiveDateTime::from_proto)
        .transpose()?,
    })
  }
}

impl ToProto for User {
  type ProtoType = crate::grpc::proto::User;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized,
  {
    Ok(crate::grpc::proto::User {
      id: self.id.get(),
      created_at: self.created_at.to_string(),
      name: self.name,
      email: self.email,
      display_name: self.display_name,
      password_hash: self.password_hash,
      updated_at: self.updated_at.map(|v| v.to_string()),
    })
  }
}

impl FromProto for Secret {
  type ProtoType = proto::Secret;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized,
  {
    Ok(Self { id: Id::from_proto(proto.id)?, jwt: proto.jwt.into() })
  }
}

impl ToProto for Secret {
  type ProtoType = proto::Secret;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized,
  {
    Ok(proto::Secret { id: self.id.get(), jwt: self.jwt.into_inner() })
  }
}

impl FromProto for chrono::NaiveDateTime {
  type ProtoType = String;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized,
  {
    proto.parse().into_capwat_error()
  }
}

impl<T: Marker> FromProto for Id<T> {
  type ProtoType = u64;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized,
  {
    if let Some(value) = Self::new_checked(proto) {
      Ok(value)
    } else {
      // Oops! Conversion error :)
      Err(crate::Error::new(crate::error::ErrorCategory::Internal))
    }
  }
}

impl<T: Marker> ToProto for Id<T> {
  type ProtoType = u64;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized,
  {
    Ok(self.get())
  }
}

impl<T: FromProto> FromProto for Sensitive<T> {
  type ProtoType = T::ProtoType;

  fn from_proto(proto: Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized,
  {
    T::from_proto(proto).map(Sensitive::new)
  }
}

impl<T: ToProto> ToProto for Sensitive<T> {
  type ProtoType = T::ProtoType;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized,
  {
    self.into_inner().to_proto()
  }
}

impl<'a> FromRefProto<'a> for drivers::data::types::CreateUser<'a> {
  type ProtoType = proto::CreateUserRequest;

  fn from_ref_proto(proto: &'a Self::ProtoType) -> crate::Result<Self>
  where
    Self: Sized + 'a,
  {
    Ok(Self {
      name: Sensitive::new(&proto.name),
      email: proto.email.as_deref().into(),
      password_hash: Sensitive::new(&proto.password_hash),
    })
  }
}

impl ToProto for drivers::data::types::CreateUser<'_> {
  type ProtoType = proto::CreateUserRequest;

  fn to_proto(self) -> crate::Result<Self::ProtoType>
  where
    Self: Sized,
  {
    Ok(proto::CreateUserRequest {
      name: self.name.into_inner().to_string(),
      email: self.email.into_inner().map(std::string::ToString::to_string),
      password_hash: self.password_hash.into_inner().to_string(),
    })
  }
}
