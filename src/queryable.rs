use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{key_expr as zkey_expr, query as zquery, session as zsession};

use crate::{config::EntityGlobalId, error::MapNapiErr, key_expr::KeyExpr};

#[napi]
pub struct Queryable {
  inner: zquery::Queryable<()>,
}

impl Queryable {
  pub(crate) fn new(inner: zquery::Queryable<()>) -> Self {
    Self { inner }
  }
}

#[napi]
impl Queryable {
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.inner.id().into()
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into()
  }

  #[napi(getter)]
  pub fn handler(&self) -> Either<(), ()> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    todo!()
  }
}
