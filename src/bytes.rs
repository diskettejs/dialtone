use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::bytes as zbytes;

use crate::error::*;

#[napi]
pub type PayloadArg = napi::Either<String, Uint8Array>;

pub(crate) trait IntoZBytes {
  fn into_zbytes(self) -> zbytes::ZBytes;
}

impl IntoZBytes for PayloadArg {
  fn into_zbytes(self) -> zbytes::ZBytes {
    match self {
      napi::Either::A(s) => zbytes::ZBytes::from(s),
      napi::Either::B(bytes) => zbytes::ZBytes::from(bytes.to_vec()),
    }
  }
}

impl IntoZBytes for Uint8Array {
  fn into_zbytes(self) -> zbytes::ZBytes {
    zbytes::ZBytes::from(self.to_vec())
  }
}

impl IntoZBytes for &Uint8Array {
  fn into_zbytes(self) -> zbytes::ZBytes {
    zbytes::ZBytes::from(self.to_vec())
  }
}

#[napi]
pub struct Bytes {
  inner: zbytes::ZBytes,
}

impl From<zbytes::ZBytes> for Bytes {
  fn from(inner: zbytes::ZBytes) -> Self {
    Self { inner }
  }
}

impl From<&Bytes> for zbytes::ZBytes {
  fn from(value: &Bytes) -> Self {
    value.inner.clone()
  }
}

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

  #[napi]
  pub fn to_string(&self) -> napi::Result<String> {
    self
      .inner
      .try_to_string()
      .map(|s| s.into_owned())
      .map_napi_err()
  }
}
