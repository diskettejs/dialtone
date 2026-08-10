use core::time::Duration;

pub(crate) trait MapNapiErr<T> {
  fn map_napi_err(self) -> napi::Result<T>;
}

impl<T, E: core::fmt::Display> MapNapiErr<T> for core::result::Result<T, E> {
  fn map_napi_err(self) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(e.to_string()))
  }
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
  pub fn get(&self) -> napi::Result<&T> {
    self.0.as_ref().ok_or_else(Self::gone)
  }

  pub fn take(&mut self) -> napi::Result<T> {
    self.0.take().ok_or_else(Self::gone)
  }

  fn gone() -> napi::Error {
    let path = core::any::type_name::<T>();
    let name = path.split('<').next().unwrap_or(path);
    let name = name.rsplit("::").next().unwrap_or(path);
    napi::Error::from_reason(format!("{name} has already been consumed."))
  }
}

/// Converts a JS millisecond timeout into a `Duration`, surfacing invalid values as errors.
pub(crate) fn duration_ms(ms: Option<f64>) -> napi::Result<Option<Duration>> {
  ms.map(|ms| Duration::try_from_secs_f64(ms / 1000.0).map_napi_err())
    .transpose()
}

/// Builds the channel backing a declaration, falling back to Zenoh's own default capacity.
pub(crate) fn fifo(capacity: Option<u32>) -> zenoh::handlers::FifoChannel {
  match capacity {
    Some(capacity) => zenoh::handlers::FifoChannel::new(capacity as usize),
    None => zenoh::handlers::FifoChannel::default(),
  }
}
