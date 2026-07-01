use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::{key_expr as zkey_expr, query as zquery};

use crate::{error::*, instance::Instance, key_expr::*, macros::wrapper, options::IntoZenoh, query::*};

wrapper!(zquery::Selector<'static>);

impl Selector {
  /// Builds the effective `zenoh` selector from a selector argument and an optional parameters
  /// override, mirroring how the session get merges parameters onto the selector's key expression.
  pub(crate) fn resolve(
    selector: SelectorArg<'_>,
    parameters: Option<Instance<Parameters>>,
  ) -> napi::Result<zquery::Selector<'static>> {
    let mut selector = zquery::Selector::from(Selector::try_from(selector)?);
    if let Some(parameters) = parameters {
      let key_expr = selector.key_expr().clone().into_owned();
      selector = zquery::Selector::owned(key_expr, parameters.into_zenoh());
    }
    Ok(selector)
  }
}

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
