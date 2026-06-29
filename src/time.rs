use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::time as ztime;

#[napi]
pub struct Timestamp {
  inner: ztime::Timestamp,
}

impl From<ztime::Timestamp> for Timestamp {
  fn from(inner: ztime::Timestamp) -> Self {
    Self { inner }
  }
}

impl From<&Timestamp> for ztime::Timestamp {
  fn from(value: &Timestamp) -> Self {
    value.inner
  }
}

#[napi]
impl Timestamp {
  #[napi(factory)]
  pub fn parse_rfc3339(s: String) -> napi::Result<Self> {
    ztime::Timestamp::parse_rfc3339(&s)
      .map(Self::from)
      .map_err(|e| napi::Error::from_reason(e.cause))
  }

  #[napi]
  pub fn to_string_rfc3339_lossy(&self) -> String {
    self.inner.to_string_rfc3339_lossy()
  }

  #[napi]
  pub fn get_time(&self) -> BigInt {
    BigInt::from(self.inner.get_time().as_u64())
  }

  #[napi]
  pub fn get_id(&self) -> String {
    self.inner.get_id().to_string()
  }

  #[napi]
  pub fn get_diff_duration(&self, other: &Timestamp) -> f64 {
    self.inner.get_diff_duration(&other.inner).as_secs_f64() * 1000.0
  }
}
