use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::bytes::ZBytes;
use zenoh_ext as ext;

use crate::bytes::Bytes;
use crate::error::*;
