use std::ops::Deref;
use std::ptr;

use napi::bindgen_prelude::*;

pub(crate) trait MapNapiErr<T> {
  fn map_napi_err(self) -> napi::Result<T>;
}

impl<T, E: std::fmt::Display> MapNapiErr<T> for std::result::Result<T, E> {
  fn map_napi_err(self) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

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

/// Owned, `Send` replacement for `ClassInstance<'a, N>` in options structs.
///
/// `ClassInstance<'env, N>` borrows the JS object: it carries the `'env` lifetime and is
/// `!Send`, which pins any options struct holding one to the JS thread and forces an `'a`
/// parameter through every signature. `Instance<N>` instead unwraps the `#[napi]` class
/// once, on the JS thread during argument conversion, and clones the native `N` out. It
/// owns `N`, so it is `Send` whenever `N` is and carries no lifetime. `Deref`/`AsRef<N>`
/// make it a drop-in for `ClassInstance<N>` at call sites.
pub struct Instance<N: Clone + 'static> {
  inner: N,
}

impl<N: Clone + 'static> Instance<N> {
  pub fn into_inner(self) -> N {
    self.inner
  }
}

impl<N: Clone + 'static> FromNapiValue for Instance<N> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let mut value = ptr::null_mut();
    check_status!(
      unsafe { sys::napi_unwrap(env, napi_val, &mut value) },
      "Failed to unwrap `{}` class instance",
      std::any::type_name::<N>(),
    )?;
    // `napi_unwrap` only borrows: ownership of the box stays with the JS object's finalizer.
    let inner = unsafe { &*(value as *const N) }.clone();
    Ok(Self { inner })
  }
}

impl<N: Clone + 'static> Deref for Instance<N> {
  type Target = N;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl<N: Clone + 'static> AsRef<N> for Instance<N> {
  fn as_ref(&self) -> &N {
    &self.inner
  }
}
