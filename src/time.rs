use derive_more::From;
use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

/// A point in time produced by the hybrid logical clock of a Zenoh session.
///
/// It pairs a 64-bit time with the id of the clock that produced it, so timestamps coming
/// from different sessions can be told apart. In string form the two are joined by a `/`,
/// as in `2024-07-01T13:51:12.129693000Z/33`.
#[napi]
#[derive(Clone, From)]
pub struct Timestamp(zenoh::time::Timestamp);

#[napi]
impl Timestamp {
  /// Parses a timestamp from its RFC 3339 form, e.g.
  /// `2024-07-01T13:51:12.129693000Z/33`.
  ///
  /// @throws If the string is not a valid RFC 3339 timestamp followed by a clock id.
  #[napi(factory)]
  pub fn parse_rfc3339(s: String) -> napi::Result<Self> {
    zenoh::time::Timestamp::parse_rfc3339(&s)
      .map(Self::from)
      .map_err(|e| napi::Error::from_reason(e.cause))
  }

  /// Returns this timestamp in RFC 3339 form with nanosecond precision, e.g.
  /// `2024-07-01T13:51:12.129693000Z/33`.
  ///
  /// The conversion is lossy: the fraction of a second is rounded to nanoseconds, so
  /// parsing the result back may not yield this exact timestamp. Use
  /// {@link Timestamp.getTime} for a lossless value.
  #[napi]
  pub fn to_string_rfc3339_lossy(&self) -> String {
    self.0.to_string_rfc3339_lossy()
  }

  /// Returns the 64-bit time of this timestamp.
  ///
  /// Its upper 32 bits are the number of seconds since the UNIX epoch and its lower 32
  /// bits are the fraction of a second, whose last few bits carry the logical counter of
  /// the clock.
  #[napi]
  pub fn get_time(&self) -> BigInt {
    BigInt::from(self.0.get_time().as_u64())
  }

  /// Returns the hexadecimal id of the clock that produced this timestamp.
  #[napi]
  pub fn get_id(&self) -> String {
    self.0.get_id().to_string()
  }

  /// Returns the time elapsed from `other` to this timestamp, in milliseconds.
  #[napi]
  pub fn get_diff_duration(&self, other: &Timestamp) -> f64 {
    self.0.get_diff_duration(&other.0).as_secs_f64() * 1000.0
  }
}
