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
