use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{Wait, query as zquery};

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
  pub fn undeclare(&mut self) -> napi::Result<()> {
    Wait::wait(self.take()?.undeclare()).map_napi_err()
  }
}
