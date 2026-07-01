use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{key_expr as zkey_expr, query as zquery};

use crate::{error::*, key_expr::*, macros::wrapper, query::*};

wrapper!(zquery::Selector<'static>);

#[napi(object)]
pub struct SelectorParts {
  pub key_expr: String,
  pub parameters: String,
}

#[napi]
impl Selector {
  #[napi(constructor)]
  pub fn new(key_expr: KeyExprArg, parameters: Option<String>) -> napi::Result<Self> {
    let key_expr = KeyExpr::try_from(key_expr)?;

    let parameters = parameters.map(zquery::Parameters::from).unwrap_or_default();

    let inner = zquery::Selector::owned(key_expr, parameters);

    Ok(inner.into())
  }

  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.inner.key_expr().clone().into_owned().into()
  }

  #[napi(getter)]
  pub fn parameters(&self) -> Parameters {
    self.inner.parameters().clone().into_owned().into()
  }

  #[napi]
  pub fn split(&self) -> SelectorParts {
    SelectorParts {
      key_expr: self.inner.key_expr().as_str().to_string(),
      parameters: self.inner.parameters().as_str().to_string(),
    }
  }
}

#[napi]
pub type SelectorArg<'a> = Either3<String, &'a KeyExpr, &'a Selector>;

impl TryFrom<SelectorArg<'_>> for Selector {
  type Error = napi::Error;

  fn try_from(value: SelectorArg<'_>) -> napi::Result<Self> {
    match value {
      Either3::A(selector) => zquery::Selector::try_from(selector)
        .map(Selector::from)
        .map_napi_err(),
      Either3::B(key_expr) => {
        Ok(zquery::Selector::from(zkey_expr::KeyExpr::from(key_expr.clone())).into())
      }
      Either3::C(selector) => Ok(zquery::Selector::from(selector).into()),
    }
  }
}
