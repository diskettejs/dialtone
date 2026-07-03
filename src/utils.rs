use napi::bindgen_prelude::{FromNapiValue, JsValue, Result, ValidateNapiValue};

/// Checked, type-directed downcasting for napi's dynamic JS values — the napi-rs
/// counterpart of PyO3's `PyAny` extraction API.
///
/// napi's [`Unknown`](napi::bindgen_prelude::Unknown) represents "any JS value",
/// the way PyO3's `PyAny` represents "any Python object", yet it only exposes an
/// `unsafe`, *unchecked* `cast`. This trait restores the checked ergonomics of
/// `PyAny`: validate a value's runtime type first — for a `#[napi]` class that is a
/// real `instanceof` test — and only then convert.
///
/// It is implemented for every napi value through a blanket impl and depends on
/// nothing in this crate, so it can be reused verbatim in other napi-rs projects.
pub trait Downcast<'env>: JsValue<'env> {
  /// Checked conversion into `T` — the analog of `PyAny::extract::<T>()`.
  ///
  /// `T` may be anything that is both [`FromNapiValue`] and [`ValidateNapiValue`]:
  /// a primitive (`String`, `u32`, …), a `#[napi]` class reference (`&MyClass`), a
  /// [`ClassInstance<T>`](napi::bindgen_prelude::ClassInstance), an
  /// [`Either`](napi::Either), and so on. Fails with `InvalidArg` when the value's
  /// runtime type is not `T`.
  fn extract<T>(&self) -> Result<T>
  where
    T: FromNapiValue + ValidateNapiValue,
  {
    let env = self.value().env;
    let value = self.raw();
    unsafe {
      <T as ValidateNapiValue>::validate(env, value)?;
      <T as FromNapiValue>::from_napi_value(env, value)
    }
  }

  /// Checked conversion that yields `Ok(None)` on a type mismatch instead of an
  /// error — the analog of `PyAny::downcast::<T>().ok()`.
  ///
  /// A mere type mismatch becomes `Ok(None)`; only a genuine napi failure surfaces
  /// as `Err`. This is the primitive for "try each type in turn" dispatch.
  fn downcast<T>(&self) -> Result<Option<T>>
  where
    T: FromNapiValue + ValidateNapiValue,
  {
    let env = self.value().env;
    let value = self.raw();
    if unsafe { <T as ValidateNapiValue>::validate(env, value) }.is_err() {
      return Ok(None);
    }
    unsafe { <T as FromNapiValue>::from_napi_value(env, value) }.map(Some)
  }

  /// Whether the value satisfies `T`'s validation — for a `#[napi]` class this is
  /// `value instanceof T`. The analog of `PyAny::is_instance_of::<T>()`.
  ///
  /// A failed validation, including a `T` whose class was never registered, reads
  /// as `false`.
  fn is_instance_of<T>(&self) -> bool
  where
    T: ValidateNapiValue,
  {
    let env = self.value().env;
    let value = self.raw();
    unsafe { <T as ValidateNapiValue>::validate(env, value) }.is_ok()
  }
}

impl<'env, V: JsValue<'env>> Downcast<'env> for V {}

pub(crate) trait IntoZenoh: 'static {
  type Into;

  fn into_zenoh(self) -> Self::Into;
}
