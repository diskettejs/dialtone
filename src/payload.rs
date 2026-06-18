use napi::bindgen_prelude::*;

/// Convert a JS payload (`string | Uint8Array`) into Zenoh bytes. Strings are
/// UTF-8 encoded; typed arrays and Node `Buffer`s (themselves `Uint8Array`s) are
/// taken as raw bytes. The JS wrapper widens `ArrayBuffer` to `Uint8Array` before
/// it reaches here, since an `ArrayBuffer` borrows the V8 scope and can't cross the
/// async boundary directly.
pub(crate) fn to_zbytes(input: Either<String, Uint8Array>) -> zenoh::bytes::ZBytes {
  match input {
    Either::A(s) => zenoh::bytes::ZBytes::from(s),
    Either::B(bytes) => zenoh::bytes::ZBytes::from(bytes.to_vec()),
  }
}
