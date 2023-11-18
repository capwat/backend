use actix_web::{dev::Payload, web::BytesMut, FromRequest};
use futures::Future;
use serde::{de::DeserializeOwned, Deserialize};
use std::{marker::PhantomData, pin::Pin};
use validator::Validate;

use crate::Result;

pub struct Json<T>(T);

const DEFAULT_LIMIT: usize = 2_097_152; // 2 mb

impl<'de, T: Deserialize<'de> + Validate> FromRequest for Json<T> {
  type Error = crate::Error;
  type Future = futures::future::LocalBoxFuture<'static, crate::Result<Self>>;

  fn from_request(
    req: &actix_web::HttpRequest,
    payload: &mut actix_web::dev::Payload,
  ) -> Self::Future {
    todo!()
  }
}

enum JsonFut<T> {
  Body {
    buf: BytesMut,
    phantom: PhantomData<T>,
    payload: Payload,
  },
}

impl<T> Unpin for JsonFut<T> {}

impl<T: DeserializeOwned + Validate> Future for JsonFut<T> {
  type Output = Result<T>;

  fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
    todo!()
  }
}
