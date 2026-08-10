use derive_more::From;
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

#[napi]
#[derive(Clone, From)]
pub struct Timestamp(zenoh::time::Timestamp);

#[napi]
impl Timestamp {
  #[napi(factory)]
  pub fn parse_rfc3339(s: String) -> napi::Result<Self> {
    zenoh::time::Timestamp::parse_rfc3339(&s)
      .map(Self::from)
      .map_err(|e| napi::Error::from_reason(e.cause))
  }

  #[napi]
  pub fn to_string_rfc3339_lossy(&self) -> String {
    self.0.to_string_rfc3339_lossy()
  }

  #[napi]
  pub fn get_time(&self) -> BigInt {
    BigInt::from(self.0.get_time().as_u64())
  }

  #[napi]
  pub fn get_id(&self) -> String {
    self.0.get_id().to_string()
  }

  #[napi]
  pub fn get_diff_duration(&self, other: &Timestamp) -> f64 {
    self.0.get_diff_duration(&other.0).as_secs_f64() * 1000.0
  }
}
