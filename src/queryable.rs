use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{handlers as zhandlers, key_expr as zkey_expr, query as zquery, session as zsession};

use crate::{config::EntityGlobalId, error::MapNapiErr, key_expr::KeyExpr, query::Query};

type ZQueryable = zquery::Queryable<()>;

#[napi]
pub struct Queryable {
  id: zsession::EntityGlobalId,
  key_expr: zkey_expr::KeyExpr<'static>,
  inner: Option<ZQueryable>,
  receiver: crate::handlers::FifoChannelHandler<zquery::Query>,
}

impl Queryable {
  pub(crate) fn new(
    inner: ZQueryable,
    receiver: zhandlers::FifoChannelHandler<zquery::Query>,
  ) -> Self {
    Self {
      id: inner.id(),
      key_expr: inner.key_expr().clone(),
      inner: Some(inner),
      receiver: receiver.into(),
    }
  }
}

#[napi]
impl Queryable {
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.id.into()
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.key_expr.clone().into()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Query> {
    self.receiver.recv::<Query>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Query>> {
    self.receiver.try_recv::<Query>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<Query> {
    self.receiver.drain::<Query>()
  }

  #[napi]
  pub fn is_disconnected(&self) -> bool {
    self.receiver.is_disconnected()
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.receiver.is_empty()
  }

  #[napi]
  pub fn is_full(&self) -> bool {
    self.receiver.is_full()
  }

  #[napi]
  pub fn len(&self) -> u32 {
    self.receiver.len()
  }

  #[napi]
  pub fn capacity(&self) -> Option<u32> {
    self.receiver.capacity()
  }

  #[napi]
  pub fn sender_count(&self) -> u32 {
    self.receiver.sender_count()
  }

  #[napi]
  pub fn receiver_count(&self) -> u32 {
    self.receiver.receiver_count()
  }

  #[napi]
  pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, Query>> {
    self.receiver.stream::<Query>(env)
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let queryable = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("queryable has already been undeclared"))?;

    env.spawn_future(async move { queryable.undeclare().await.map_napi_err() })
  }
}
