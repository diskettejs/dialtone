use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::bytes as zbytes;

use crate::{error::*, macros::wrapper, options::IntoZenoh};

#[napi]
pub type Payload = napi::Either<String, Uint8Array>;

pub(crate) trait IntoZBytes {
  fn into_zbytes(self) -> zbytes::ZBytes;
}

impl IntoZBytes for Payload {
  fn into_zbytes(self) -> zbytes::ZBytes {
    match self {
      napi::Either::A(s) => zbytes::ZBytes::from(s),
      napi::Either::B(bytes) => zbytes::ZBytes::from(bytes.to_vec()),
    }
  }
}

impl IntoZenoh for Payload {
  type Into = zenoh::bytes::ZBytes;
  fn into_zenoh(self) -> zenoh::bytes::ZBytes {
    self.into_zbytes()
  }
}

wrapper!(zbytes::ZBytes as Bytes);

#[napi]
impl Bytes {
  #[napi]
  pub fn new() -> Self {
    Self {
      inner: zbytes::ZBytes::new(),
    }
  }

  #[napi(factory)]
  pub fn from_bytes(data: Uint8Array) -> Self {
    zbytes::ZBytes::from(data.to_vec()).into()
  }

  #[napi(factory)]
  pub fn from_string(value: String) -> Self {
    zbytes::ZBytes::from(value).into()
  }

  #[napi(getter)]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  #[napi(getter)]
  pub fn len(&self) -> u32 {
    self.inner.len() as u32
  }

  #[napi]
  pub fn to_bytes(&self) -> Uint8Array {
    Uint8Array::from(self.inner.to_bytes().into_owned())
  }

  /// Decodes the payload as a UTF-8 string.
  ///
  /// @throws If the payload contains non-UTF-8 bytes. Zenoh does not guarantee
  /// UTF-8, so this can fail; use {@link Bytes.toBytes} for arbitrary bytes.
  #[napi]
  pub fn try_to_string(&self) -> napi::Result<String> {
    self
      .inner
      .try_to_string()
      .map(|s| s.into_owned())
      .map_napi_err()
  }
}
