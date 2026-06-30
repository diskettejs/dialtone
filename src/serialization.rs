use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::bytes::ZBytes;
use zenoh_ext as ext;

use crate::bytes::Bytes;
use crate::error::*;

#[napi]
pub struct Serializer {
  inner: Option<ext::ZSerializer>,
}

impl Serializer {
  fn writer(&mut self) -> napi::Result<&mut ext::ZSerializer> {
    self
      .inner
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("serializer already finished"))
  }
}

fn range_err(ty: &str) -> napi::Error {
  napi::Error::from_reason(format!("BigInt value out of range for {ty}"))
}

fn bigint_to_i128(value: &BigInt) -> Option<i128> {
  if value.words.len() > 2 {
    return None;
  }
  let mag = (value.words[0] as u128) | ((value.words.get(1).copied().unwrap_or(0) as u128) << 64);
  if value.sign_bit {
    if mag == i128::MIN.unsigned_abs() {
      Some(i128::MIN)
    } else {
      (mag <= i128::MAX as u128).then(|| -(mag as i128))
    }
  } else {
    (mag <= i128::MAX as u128).then_some(mag as i128)
  }
}

#[napi]
impl Serializer {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      inner: Some(ext::ZSerializer::new()),
    }
  }

  #[napi]
  pub fn i8(&mut self, value: i8) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn i16(&mut self, value: i16) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn i32(&mut self, value: i32) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn i64(&mut self, value: BigInt) -> napi::Result<()> {
    let (n, lossless) = value.get_i64();
    if !lossless {
      return Err(range_err("i64"));
    }
    self.writer()?.serialize(n);
    Ok(())
  }

  #[napi]
  pub fn i128(&mut self, value: BigInt) -> napi::Result<()> {
    let n = bigint_to_i128(&value).ok_or_else(|| range_err("i128"))?;
    self.writer()?.serialize(n);
    Ok(())
  }

  #[napi]
  pub fn u8(&mut self, value: u8) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn u16(&mut self, value: u16) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn u32(&mut self, value: u32) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn u64(&mut self, value: BigInt) -> napi::Result<()> {
    let (signed, n, lossless) = value.get_u64();
    if signed || !lossless {
      return Err(range_err("u64"));
    }
    self.writer()?.serialize(n);
    Ok(())
  }

  #[napi]
  pub fn u128(&mut self, value: BigInt) -> napi::Result<()> {
    let (signed, n, lossless) = value.get_u128();
    if signed || !lossless {
      return Err(range_err("u128"));
    }
    self.writer()?.serialize(n);
    Ok(())
  }

  #[napi]
  pub fn f64(&mut self, value: f64) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn var_int(&mut self, value: BigInt) -> napi::Result<()> {
    let (signed, n, lossless) = value.get_u64();
    if signed || !lossless {
      return Err(range_err("varint"));
    }
    let n = usize::try_from(n).map_err(|_| range_err("varint"))?;
    self.writer()?.serialize(ext::VarInt(n));
    Ok(())
  }

  #[napi]
  pub fn bool(&mut self, value: bool) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn string(&mut self, value: String) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn bytes(&mut self, value: &Bytes) -> napi::Result<()> {
    self.writer()?.serialize(ZBytes::from(value));
    Ok(())
  }

  #[napi]
  pub fn uint8_array(&mut self, value: Uint8Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn int8_array(&mut self, value: Int8Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn uint16_array(&mut self, value: Uint16Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn int16_array(&mut self, value: Int16Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn uint32_array(&mut self, value: Uint32Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn int32_array(&mut self, value: Int32Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn float32_array(&mut self, value: Float32Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn float64_array(&mut self, value: Float64Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn big_int64_array(&mut self, value: BigInt64Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn big_uint64_array(&mut self, value: BigUint64Array) -> napi::Result<()> {
    self.writer()?.serialize(value.to_vec());
    Ok(())
  }

  #[napi]
  pub fn string_array(&mut self, value: Vec<String>) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn bool_array(&mut self, value: Vec<bool>) -> napi::Result<()> {
    self.writer()?.serialize(value);
    Ok(())
  }

  #[napi]
  pub fn finish(&mut self) -> napi::Result<Bytes> {
    self
      .inner
      .take()
      .map(|s| s.finish().into())
      .ok_or_else(|| napi::Error::from_reason("serializer already finished"))
  }
}

#[napi]
pub struct Deserializer {
  inner: SharedReference<Bytes, ext::ZDeserializer<'static>>,
}

#[napi]
impl Deserializer {
  #[napi(constructor)]
  pub fn new(bytes: Reference<Bytes>, env: Env) -> napi::Result<Self> {
    Ok(Self {
      inner: bytes.share_with(env, |b| Ok(ext::ZDeserializer::new(b.as_zbytes())))?,
    })
  }

  #[napi]
  pub fn done(&self) -> bool {
    self.inner.done()
  }

  #[napi]
  pub fn i8(&mut self) -> napi::Result<i8> {
    self.inner.deserialize::<i8>().map_napi_err()
  }

  #[napi]
  pub fn i16(&mut self) -> napi::Result<i16> {
    self.inner.deserialize::<i16>().map_napi_err()
  }

  #[napi]
  pub fn i32(&mut self) -> napi::Result<i32> {
    self.inner.deserialize::<i32>().map_napi_err()
  }

  #[napi]
  pub fn i64(&mut self) -> napi::Result<BigInt> {
    self
      .inner
      .deserialize::<i64>()
      .map(BigInt::from)
      .map_napi_err()
  }

  #[napi]
  pub fn i128(&mut self) -> napi::Result<BigInt> {
    self
      .inner
      .deserialize::<i128>()
      .map(BigInt::from)
      .map_napi_err()
  }

  #[napi]
  pub fn u8(&mut self) -> napi::Result<u8> {
    self.inner.deserialize::<u8>().map_napi_err()
  }

  #[napi]
  pub fn u16(&mut self) -> napi::Result<u16> {
    self.inner.deserialize::<u16>().map_napi_err()
  }

  #[napi]
  pub fn u32(&mut self) -> napi::Result<u32> {
    self.inner.deserialize::<u32>().map_napi_err()
  }

  #[napi]
  pub fn u64(&mut self) -> napi::Result<BigInt> {
    self
      .inner
      .deserialize::<u64>()
      .map(BigInt::from)
      .map_napi_err()
  }

  #[napi]
  pub fn u128(&mut self) -> napi::Result<BigInt> {
    self
      .inner
      .deserialize::<u128>()
      .map(BigInt::from)
      .map_napi_err()
  }

  #[napi]
  pub fn f64(&mut self) -> napi::Result<f64> {
    self.inner.deserialize::<f64>().map_napi_err()
  }

  #[napi]
  pub fn var_int(&mut self) -> napi::Result<BigInt> {
    self
      .inner
      .deserialize::<ext::VarInt<usize>>()
      .map(|v| BigInt::from(v.0 as u64))
      .map_napi_err()
  }

  #[napi]
  pub fn bool(&mut self) -> napi::Result<bool> {
    self.inner.deserialize::<bool>().map_napi_err()
  }

  #[napi]
  pub fn string(&mut self) -> napi::Result<String> {
    self.inner.deserialize::<String>().map_napi_err()
  }

  #[napi]
  pub fn bytes(&mut self) -> napi::Result<Bytes> {
    self
      .inner
      .deserialize::<Vec<u8>>()
      .map(|v| Bytes::from(ZBytes::from(v)))
      .map_napi_err()
  }

  #[napi]
  pub fn uint8_array(&mut self) -> napi::Result<Uint8Array> {
    self
      .inner
      .deserialize::<Vec<u8>>()
      .map(Uint8Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn int8_array(&mut self) -> napi::Result<Int8Array> {
    self
      .inner
      .deserialize::<Vec<i8>>()
      .map(Int8Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn uint16_array(&mut self) -> napi::Result<Uint16Array> {
    self
      .inner
      .deserialize::<Vec<u16>>()
      .map(Uint16Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn int16_array(&mut self) -> napi::Result<Int16Array> {
    self
      .inner
      .deserialize::<Vec<i16>>()
      .map(Int16Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn uint32_array(&mut self) -> napi::Result<Uint32Array> {
    self
      .inner
      .deserialize::<Vec<u32>>()
      .map(Uint32Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn int32_array(&mut self) -> napi::Result<Int32Array> {
    self
      .inner
      .deserialize::<Vec<i32>>()
      .map(Int32Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn float32_array(&mut self) -> napi::Result<Float32Array> {
    self
      .inner
      .deserialize::<Vec<f32>>()
      .map(Float32Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn float64_array(&mut self) -> napi::Result<Float64Array> {
    self
      .inner
      .deserialize::<Vec<f64>>()
      .map(Float64Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn big_int64_array(&mut self) -> napi::Result<BigInt64Array> {
    self
      .inner
      .deserialize::<Vec<i64>>()
      .map(BigInt64Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn big_uint64_array(&mut self) -> napi::Result<BigUint64Array> {
    self
      .inner
      .deserialize::<Vec<u64>>()
      .map(BigUint64Array::from)
      .map_napi_err()
  }

  #[napi]
  pub fn string_array(&mut self) -> napi::Result<Vec<String>> {
    self.inner.deserialize::<Vec<String>>().map_napi_err()
  }

  #[napi]
  pub fn bool_array(&mut self) -> napi::Result<Vec<bool>> {
    self.inner.deserialize::<Vec<bool>>().map_napi_err()
  }
}
