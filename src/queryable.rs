use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{handlers as zhandlers, key_expr as zkey_expr, query as zquery, session as zsession};

use crate::{
  config::EntityGlobalId, error::MapNapiErr, key_expr::KeyExpr, macros::channel_forward,
  query::Query,
};

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
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let queryable = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("queryable has already been undeclared"))?;

    env.spawn_future(async move { queryable.undeclare().await.map_napi_err() })
  }
}

channel_forward!(Queryable, receiver, Query);
