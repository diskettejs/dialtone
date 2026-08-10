use derive_more::{From, Into};
use napi_derive::napi;
use zenoh as z;

use crate::utils::MapNapiErr;

/// A set of keys, expressed with a glob-like syntax.
///
/// A key expression is a `/`-separated list of non-empty UTF-8 chunks. It may never start
/// or end with `/`, contain `//`, or contain any of the characters `#$?` outside of the
/// `$*` wildcard. It must also be in canonical form, so that two key expressions denoting
/// the same set are always the same string.
///
/// Three wildcards widen the set a key expression denotes: `*` stands for a single chunk,
/// `**` for any number of chunks, and `$*` for any substring within a chunk.
///
/// Since key expressions denote sets, they relate to one another: two of them
/// {@link KeyExpr.intersects} if they share at least one key, and one of them
/// {@link KeyExpr.includes} another if it holds every key of the other. Two key
/// expressions denote the same set exactly when they are the same string.
#[napi]
#[derive(Clone, From, Into)]
pub struct KeyExpr(z::key_expr::KeyExpr<'static>);

/// A {@link `KeyExpr`} or the string to build one from.
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
  /// Creates a key expression from its string form.
  ///
  /// @throws If the string is not a valid key expression. Note that being valid requires
  /// being canonical; use {@link KeyExpr.autocanonize} to canonize it first.
  #[napi(constructor)]
  pub fn new(expr: String) -> napi::Result<Self> {
    let inner = z::key_expr::KeyExpr::new(expr).map_napi_err()?;
    Ok(inner.into())
  }

  /// Canonizes the given string, then creates a key expression from it.
  ///
  /// @throws If the string is not a valid key expression even after canonization.
  #[napi(factory)]
  pub fn autocanonize(expr: String) -> napi::Result<Self> {
    let inner = z::key_expr::KeyExpr::autocanonize(expr).map_napi_err()?;
    Ok(inner.into())
  }

  /// Creates a key expression from its string form, same as the {@link `KeyExpr`}
  /// constructor.
  ///
  /// @throws If the string is not a valid, canonical key expression.
  #[napi(factory)]
  pub fn from_str(expr: String) -> napi::Result<Self> {
    Self::new(expr)
  }

  /// Appends `other` to this key expression without inserting a separator.
  ///
  /// Prefer {@link KeyExpr.join}, as Zenoh can take advantage of the hierarchical
  /// separation it inserts.
  ///
  /// @throws If the result is not a valid key expression, or if this key expression ends
  /// with `*` while `other` starts with `*`.
  #[napi]
  pub fn concat(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.0.concat(&other).map_napi_err()?;
    Ok(inner.into())
  }

  /// Joins this key expression and `other`, inserting a `/` in between them.
  ///
  /// This is the preferred way of concatenating path segments.
  ///
  /// @throws If the result is not a valid key expression.
  #[napi]
  pub fn join(&self, other: String) -> napi::Result<KeyExpr> {
    let inner = self.0.join(&other).map_napi_err()?;
    Ok(inner.into())
  }

  /// Returns this key expression in its string form.
  #[napi]
  #[allow(clippy::inherent_to_string)]
  pub fn to_string(&self) -> String {
    self.0.as_str().to_string()
  }

  /// Returns `true` if the two key expressions intersect, i.e. if at least one key belongs
  /// to the sets denoted by both of them.
  #[napi]
  pub fn intersects(&self, other: &KeyExpr) -> bool {
    self.0.as_keyexpr().intersects(other.0.as_keyexpr())
  }

  /// Returns `true` if this key expression includes `other`, i.e. if the set it denotes
  /// holds every key of the set denoted by `other`.
  #[napi]
  pub fn includes(&self, other: &KeyExpr) -> bool {
    self.0.as_keyexpr().includes(other.0.as_keyexpr())
  }

  /// Whether this key expression contains a wildcard.
  #[napi(getter)]
  pub fn is_wild(&self) -> bool {
    self.0.as_keyexpr().is_wild()
  }
}
