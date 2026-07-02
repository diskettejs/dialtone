use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::query as zquery;

use crate::{
  config::EntityGlobalId, error::MapNapiErr, handlers::HandlerImpl, key_expr::KeyExpr,
  macros::option_wrapper, query::Query,
};

option_wrapper!(
  zquery::Queryable<HandlerImpl<zquery::Query>>,
  "Undeclared queryable"
);

#[napi]
impl Queryable {
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.get_ref()?.id().into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.get_ref()?.key_expr().clone().into())
  }

  #[napi(getter)]
  pub fn handler(&self) -> napi::Result<()> {
    todo!()
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let queryable = self.take()?;

    env.spawn_future(async move { queryable.undeclare().await.map_napi_err() })
  }
}
