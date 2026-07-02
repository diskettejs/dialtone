#![allow(dead_code)]
use napi::Unknown;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh_ext as ext;

use crate::bytes::Bytes;
use crate::error::*;

/// Internal shape of a `Schema`. Rust owns what a valid node looks like, so there
/// is no JS-defined shape to re-validate. `Object` keeps declaration order because
/// that order is the cross-language wire contract (an object is a Rust tuple on the
/// wire, matched positionally by peers).
#[derive(Clone)]
enum SchemaNode {
  Bool,
  String,
  Bytes,
  Int8,
  Int16,
  Int32,
  UInt8,
  UInt16,
  UInt32,
  Int64,
  UInt64,
  Float32,
  Float64,
  Array(Box<SchemaNode>),
  Tuple(Vec<SchemaNode>),
  Object(Vec<(String, SchemaNode)>),
  Map(Box<SchemaNode>, Box<SchemaNode>),
  Set(Box<SchemaNode>),
}

/// An immutable schema value: build once with the `zd.*` builders, then call
/// `serialize`/`deserialize` as many times as needed. No lifecycle.
#[napi(namespace = "zd")]
pub struct Schema {
  node: SchemaNode,
}

#[napi(namespace = "zd")]
impl Schema {
  #[napi]
  pub fn serialize(&self, data: Unknown) -> napi::Result<Bytes> {
    let mut ser = ext::ZSerializer::new();
    write(&self.node, data, &mut ser)?;
    Ok(ser.finish().into())
  }

  #[napi]
  pub fn deserialize<'env>(&self, env: &'env Env, bytes: &Bytes) -> napi::Result<Unknown<'env>> {
    let mut de = ext::ZDeserializer::new(bytes.as_ref());
    let value = read(&self.node, env, &mut de)?;
    if !de.done() {
      return Err(Error::from_reason(
        "trailing bytes: schema and payload disagree",
      ));
    }
    Ok(value)
  }
}

#[napi(namespace = "zd")]
pub fn bool() -> Schema {
  Schema {
    node: SchemaNode::Bool,
  }
}

#[napi(namespace = "zd")]
pub fn string() -> Schema {
  Schema {
    node: SchemaNode::String,
  }
}

#[napi(namespace = "zd")]
pub fn bytes() -> Schema {
  Schema {
    node: SchemaNode::Bytes,
  }
}

#[napi(namespace = "zd")]
pub fn i8() -> Schema {
  Schema {
    node: SchemaNode::Int8,
  }
}

#[napi(namespace = "zd")]
pub fn i16() -> Schema {
  Schema {
    node: SchemaNode::Int16,
  }
}

#[napi(namespace = "zd")]
pub fn i32() -> Schema {
  Schema {
    node: SchemaNode::Int32,
  }
}

#[napi(namespace = "zd")]
pub fn u8() -> Schema {
  Schema {
    node: SchemaNode::UInt8,
  }
}

#[napi(namespace = "zd")]
pub fn u16() -> Schema {
  Schema {
    node: SchemaNode::UInt16,
  }
}

#[napi(namespace = "zd")]
pub fn u32() -> Schema {
  Schema {
    node: SchemaNode::UInt32,
  }
}

#[napi(namespace = "zd")]
pub fn i64() -> Schema {
  Schema {
    node: SchemaNode::Int64,
  }
}

#[napi(namespace = "zd")]
pub fn u64() -> Schema {
  Schema {
    node: SchemaNode::UInt64,
  }
}

#[napi(namespace = "zd")]
pub fn f32() -> Schema {
  Schema {
    node: SchemaNode::Float32,
  }
}

#[napi(namespace = "zd")]
pub fn f64() -> Schema {
  Schema {
    node: SchemaNode::Float64,
  }
}

#[napi(namespace = "zd")]
pub fn array(item: &Schema) -> Schema {
  Schema {
    node: SchemaNode::Array(Box::new(item.node.clone())),
  }
}

#[napi(namespace = "zd")]
pub fn set(item: &Schema) -> Schema {
  Schema {
    node: SchemaNode::Set(Box::new(item.node.clone())),
  }
}

#[napi(namespace = "zd")]
pub fn map(key: &Schema, value: &Schema) -> Schema {
  Schema {
    node: SchemaNode::Map(Box::new(key.node.clone()), Box::new(value.node.clone())),
  }
}

#[napi(namespace = "zd")]
pub fn tuple(items: Vec<&Schema>) -> Schema {
  Schema {
    node: SchemaNode::Tuple(items.iter().map(|s| s.node.clone()).collect()),
  }
}

#[napi(namespace = "zd")]
pub fn object(fields: Object) -> napi::Result<Schema> {
  let mut out = Vec::new();
  for key in Object::keys(&fields)? {
    let schema = fields
      .get::<&Schema>(key.as_str())?
      .ok_or_else(|| Error::from_reason(format!("field `{key}` is not a Schema")))?;
    out.push((key, schema.node.clone()));
  }
  Ok(Schema {
    node: SchemaNode::Object(out),
  })
}

#[napi(namespace = "zd")]
pub fn serialize(schema: &Schema, data: Unknown) -> napi::Result<Bytes> {
  schema.serialize(data)
}

#[napi(namespace = "zd")]
pub fn deserialize<'env>(
  schema: &Schema,
  env: &'env Env,
  bytes: &Bytes,
) -> napi::Result<Unknown<'env>> {
  schema.deserialize(env, bytes)
}

fn write(node: &SchemaNode, val: Unknown, ser: &mut ext::ZSerializer) -> napi::Result<()> {
  match node {
    SchemaNode::Bool => ser.serialize(bool::from_unknown(val)?),
    SchemaNode::Int8 => ser.serialize(i8::from_unknown(val)?),
    SchemaNode::Int16 => ser.serialize(i16::from_unknown(val)?),
    SchemaNode::Int32 => ser.serialize(i32::from_unknown(val)?),
    SchemaNode::UInt8 => ser.serialize(u8::from_unknown(val)?),
    SchemaNode::UInt16 => ser.serialize(u16::from_unknown(val)?),
    SchemaNode::UInt32 => ser.serialize(u32::from_unknown(val)?),
    SchemaNode::Int64 => {
      let (v, _) = BigInt::from_unknown(val)?.get_i64();
      ser.serialize(v);
    }
    SchemaNode::UInt64 => {
      let (_, v, _) = BigInt::from_unknown(val)?.get_u64();
      ser.serialize(v);
    }
    SchemaNode::Float32 => ser.serialize(f64::from_unknown(val)? as f32),
    SchemaNode::Float64 => ser.serialize(f64::from_unknown(val)?),
    SchemaNode::String => ser.serialize(String::from_unknown(val)?.as_str()),
    SchemaNode::Bytes => {
      let arr = Uint8Array::from_unknown(val)?;
      ser.serialize(&arr[..]);
    }
    SchemaNode::Array(item) => {
      let arr = Array::from_unknown(val)?;
      let n = arr.len();
      ser.serialize(ext::VarInt(n as usize));
      for i in 0..n {
        let el: Unknown = arr
          .get(i)?
          .ok_or_else(|| Error::from_reason("array element is empty"))?;
        write(item, el, ser)?;
      }
    }
    SchemaNode::Tuple(items) => {
      let arr = Array::from_unknown(val)?;
      for (i, node) in items.iter().enumerate() {
        let el: Unknown = arr
          .get(i as u32)?
          .ok_or_else(|| Error::from_reason(format!("tuple element {i} is missing")))?;
        write(node, el, ser)?;
      }
    }
    SchemaNode::Object(fields) => {
      let obj = Object::from_unknown(val)?;
      for (name, node) in fields {
        let child: Unknown = obj.get_named_property_unchecked(name.as_str())?;
        write(node, child, ser)?;
      }
    }
    SchemaNode::Set(item) => {
      let elems = iter_values(val, "values")?;
      ser.serialize(ext::VarInt(elems.len()));
      for e in elems {
        write(item, e, ser)?;
      }
    }
    SchemaNode::Map(key, value) => {
      let entries = iter_values(val, "entries")?;
      ser.serialize(ext::VarInt(entries.len()));
      for entry in entries {
        let pair = Array::from_unknown(entry)?;
        let k: Unknown = pair
          .get(0)?
          .ok_or_else(|| Error::from_reason("map entry missing key"))?;
        let v: Unknown = pair
          .get(1)?
          .ok_or_else(|| Error::from_reason("map entry missing value"))?;
        write(key, k, ser)?;
        write(value, v, ser)?;
      }
    }
  }
  Ok(())
}

/// Decode one node into a JS value. Leaves materialize through the safe
/// constructors and hand an `Unknown` back up for the parent to place.
fn read<'env>(
  node: &SchemaNode,
  env: &'env Env,
  de: &mut ext::ZDeserializer,
) -> napi::Result<Unknown<'env>> {
  match node {
    SchemaNode::Bool => de.deserialize::<bool>().map_napi_err()?.into_unknown(env),
    SchemaNode::Int8 => (de.deserialize::<i8>().map_napi_err()? as i32).into_unknown(env),
    SchemaNode::Int16 => (de.deserialize::<i16>().map_napi_err()? as i32).into_unknown(env),
    SchemaNode::Int32 => de.deserialize::<i32>().map_napi_err()?.into_unknown(env),
    SchemaNode::UInt8 => (de.deserialize::<u8>().map_napi_err()? as u32).into_unknown(env),
    SchemaNode::UInt16 => (de.deserialize::<u16>().map_napi_err()? as u32).into_unknown(env),
    SchemaNode::UInt32 => de.deserialize::<u32>().map_napi_err()?.into_unknown(env),
    SchemaNode::Int64 => BigInt::from(de.deserialize::<i64>().map_napi_err()?).into_unknown(env),
    SchemaNode::UInt64 => BigInt::from(de.deserialize::<u64>().map_napi_err()?).into_unknown(env),
    SchemaNode::Float32 => de.deserialize::<f32>().map_napi_err()?.into_unknown(env),
    SchemaNode::Float64 => de.deserialize::<f64>().map_napi_err()?.into_unknown(env),
    SchemaNode::String => de.deserialize::<String>().map_napi_err()?.into_unknown(env),
    SchemaNode::Bytes => {
      let v = de.deserialize::<Vec<u8>>().map_napi_err()?;
      Uint8Array::from(v).into_unknown(env)
    }
    SchemaNode::Array(item) => {
      let len = de.deserialize::<ext::VarInt<usize>>().map_napi_err()?.0;
      let mut elems: Vec<Unknown> = Vec::with_capacity(len);
      for _ in 0..len {
        elems.push(read(item, env, de)?);
      }
      Array::from_vec(env, elems)?.into_unknown(env)
    }
    SchemaNode::Set(item) => {
      let len = de.deserialize::<ext::VarInt<usize>>().map_napi_err()?.0;
      let mut elems: Vec<Unknown> = Vec::with_capacity(len);
      for _ in 0..len {
        elems.push(read(item, env, de)?);
      }
      construct(env, "Set", Array::from_vec(env, elems)?)
    }
    SchemaNode::Map(key, value) => {
      let len = de.deserialize::<ext::VarInt<usize>>().map_napi_err()?.0;
      let mut pairs: Vec<Array> = Vec::with_capacity(len);
      for _ in 0..len {
        let k = read(key, env, de)?;
        let v = read(value, env, de)?;
        pairs.push(Array::from_vec(env, vec![k, v])?);
      }
      construct(env, "Map", Array::from_vec(env, pairs)?)
    }
    SchemaNode::Tuple(items) => {
      let mut elems: Vec<Unknown> = Vec::with_capacity(items.len());
      for node in items {
        elems.push(read(node, env, de)?);
      }
      Array::from_vec(env, elems)?.into_unknown(env)
    }
    SchemaNode::Object(fields) => {
      let mut obj = Object::new(env)?;
      for (name, node) in fields {
        let child = read(node, env, de)?;
        obj.set(name.as_str(), child)?;
      }
      obj.into_unknown(env)
    }
  }
}

/// Drive a JS iterable (`Set`/`Map`) through its iterator protocol, collecting each
/// yielded value. Mirrors napi's own `HashSet` conversion; used for both `.values()`
/// (sets) and `.entries()` (maps).
fn iter_values<'env>(val: Unknown<'env>, iter_name: &str) -> napi::Result<Vec<Unknown<'env>>> {
  let obj = Object::from_unknown(val)?;
  let iter_fn: Function<'_, (), Object> = obj.get_named_property(iter_name)?;
  let iter = iter_fn.apply(obj, ())?;
  let next: Function<'_, (), Object> = iter.get_named_property("next")?;
  let mut out = Vec::new();
  loop {
    let step = next.apply(iter, ())?;
    let done: bool = step.get_named_property("done")?;
    if done {
      break;
    }
    let value: Unknown = step.get_named_property_unchecked("value")?;
    out.push(value);
  }
  Ok(out)
}

/// Instantiate a global JS class (`Set`/`Map`) from an array of elements/pairs.
fn construct<'env>(env: &'env Env, class: &str, arg: Array) -> napi::Result<Unknown<'env>> {
  let global = env.get_global()?;
  let ctor: Function<'_, Array, ()> = global.get_named_property_unchecked(class)?;
  ctor.new_instance(arg)
}
