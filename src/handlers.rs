use futures::StreamExt;
use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{handlers as zhandlers, query as zquery};

use crate::{error::MapNapiErr, macros::channel_forward, query::Reply};

pub struct FifoChannelHandler<T> {
  inner: zhandlers::FifoChannelHandler<T>,
}

impl<T> From<zhandlers::FifoChannelHandler<T>> for FifoChannelHandler<T> {
  fn from(inner: zhandlers::FifoChannelHandler<T>) -> Self {
    Self { inner }
  }
}

impl<T> FifoChannelHandler<T> {
  pub async fn recv<J: From<T>>(&self) -> napi::Result<J> {
    self.inner.recv_async().await.map(J::from).map_napi_err()
  }

  pub fn try_recv<J: From<T>>(&self) -> napi::Result<Option<J>> {
    self.inner.try_recv().map(|t| t.map(J::from)).map_napi_err()
  }

  pub fn drain<J: From<T>>(&self) -> Vec<J> {
    self.inner.drain().map(J::from).collect()
  }

  pub fn is_disconnected(&self) -> bool {
    self.inner.is_disconnected()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub fn is_full(&self) -> bool {
    self.inner.is_full()
  }

  pub fn len(&self) -> u32 {
    self.inner.len() as u32
  }

  pub fn capacity(&self) -> Option<u32> {
    self.inner.capacity().map(|c| c as u32)
  }

  pub fn sender_count(&self) -> u32 {
    self.inner.sender_count() as u32
  }

  pub fn receiver_count(&self) -> u32 {
    self.inner.receiver_count() as u32
  }

  pub fn stream<'env, J>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, J>>
  where
    T: Clone + Send + 'static,
    J: From<T> + ToNapiValue + Send + 'static,
  {
    let stream = self.inner.clone().into_stream().map(|t| Ok(J::from(t)));
    ReadableStream::new(env, stream)
  }
}

#[napi]
pub struct Replies {
  inner: FifoChannelHandler<zquery::Reply>,
}

impl From<zhandlers::FifoChannelHandler<zquery::Reply>> for Replies {
  fn from(receiver: zhandlers::FifoChannelHandler<zquery::Reply>) -> Self {
    Self {
      inner: receiver.into(),
    }
  }
}

channel_forward!(Replies, inner, Reply);
