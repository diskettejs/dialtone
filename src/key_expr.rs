use derive_more::{From, Into};
use napi_derive::napi;
use zenoh as z;

use crate::utils::*;

#[napi]
#[derive(Clone, From, Into)]
pub struct KeyExpr(z::key_expr::KeyExpr<'static>);

#[napi]
pub type KeyExprArg<'a> = napi::Either<String, &'a KeyExpr>;

impl TryFrom<KeyExprArg<'_>> for KeyExpr {
  type Error = napi::Error;

  fn try_from(value: KeyExprArg<'_>) -> napi::Result<Self> {
    match value {
      napi::Either::A(expr) => KeyExpr::new(expr),
      napi::Either::B(expr) => Ok(expr.clone()),
    }
  }
}

#[napi]
impl KeyExpr {
  #[napi(constructor)]
  pub fn new(expr: String) -> napi::Result<Self> {
    let inner = z::key_expr::KeyExpr::new(expr).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn autocanonize(expr: String) -> napi::Result<Self> {
    let inner = z::key_expr::KeyExpr::autocanonize(expr).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn from_str(expr: String) -> napi::Result<Self> {
    Self::new(expr)
  }

  #[napi]
  pub fn concat(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.0.concat(&other).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi]
  pub fn join(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.0.join(&other).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  #[napi]
  pub fn intersects(&self, other: &KeyExpr) -> bool {
    self.0.as_keyexpr().intersects(other.0.as_keyexpr())
  }

  #[napi]
  pub fn includes(&self, other: &KeyExpr) -> bool {
    self.0.as_keyexpr().includes(other.0.as_keyexpr())
  }

  #[napi(getter)]
  pub fn is_wild(&self) -> bool {
    self.0.as_keyexpr().is_wild()
  }
}
