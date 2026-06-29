use napi_derive::napi;
use zenoh::key_expr as zkey_expr;

use crate::error::*;

#[derive(Clone)]
#[napi]
pub struct KeyExpr {
  inner: zkey_expr::KeyExpr<'static>,
}

impl From<zkey_expr::KeyExpr<'static>> for KeyExpr {
  fn from(inner: zkey_expr::KeyExpr<'static>) -> Self {
    Self { inner }
  }
}

impl<'a> From<KeyExpr> for zkey_expr::KeyExpr<'a> {
  fn from(value: KeyExpr) -> Self {
    value.inner
  }
}

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
    let inner = zkey_expr::KeyExpr::new(expr).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn autocanonize(expr: String) -> napi::Result<Self> {
    let inner = zkey_expr::KeyExpr::autocanonize(expr).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi(factory)]
  pub fn from_str(expr: String) -> napi::Result<Self> {
    Self::new(expr)
  }

  #[napi]
  pub fn concat(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.inner.concat(&other).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi]
  pub fn join(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.inner.join(&other).map_napi_err()?;
    Ok(inner.into())
  }

  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.inner.as_str().to_string()
  }

  #[napi]
  pub fn intersects(&self, other: &KeyExpr) -> bool {
    self.inner.as_keyexpr().intersects(other.inner.as_keyexpr())
  }

  #[napi]
  pub fn includes(&self, other: &KeyExpr) -> bool {
    self.inner.as_keyexpr().includes(other.inner.as_keyexpr())
  }

  #[napi(getter)]
  pub fn is_wild(&self) -> bool {
    self.inner.as_keyexpr().is_wild()
  }
}
