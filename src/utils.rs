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

/// Holds a Zenoh entity `T` that can be undeclared, dropped or stopped exactly once, on
/// behalf of the `#[napi]` class that exposes it.
///
/// `get` borrows the entity and `take` consumes it; both fail once the entity is gone.
pub struct Declared<T>(Option<T>);

impl<T> From<T> for Declared<T> {
  fn from(value: T) -> Self {
    Self(Some(value))
  }
}

impl<T> Declared<T> {
  pub fn get(&self) -> Result<&T> {
    self.0.as_ref().ok_or_else(Self::gone)
  }

  pub fn take(&mut self) -> Result<T> {
    self.0.take().ok_or_else(Self::gone)
  }

  fn gone() -> napi::Error {
    let path = std::any::type_name::<T>();
    let name = path.split('<').next().unwrap_or(path);
    let name = name.rsplit("::").next().unwrap_or(path);
    napi::Error::from_reason(format!("{name} has already been consumed."))
  }
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

/// Applies optional builder setters in one shot (ported from zenoh-python's `build!`).
/// Each `$value` is an `Option<T: IntoZenoh>` local whose name matches the builder setter;
/// present values are converted via `IntoZenoh` and applied, `None`s are skipped, and the
/// finished builder is returned. Conversions run synchronously as the builder is assembled,
/// so calling `build!(..).await` consumes every input before the first await point.
///
/// build!(session.put(expr, payload), encoding, priority, timestamp);
macro_rules! build {
    ($builder:expr $(, $value:ident)* $(,)?) => {{
        let mut builder = $builder;
        $(
            if let Some(value) = $value.map($crate::utils::IntoZenoh::into_zenoh) {
                builder = builder.$value(value);
            }
        )*
        builder
    }};
}
pub(crate) use build;
