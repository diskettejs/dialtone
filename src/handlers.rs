use std::marker::PhantomData;

use napi_derive::napi;

pub(crate) enum HandlerImpl<T> {
  Rust(PhantomData<T>),
}

pub(crate) trait IntoZenoh: 'static {
  type Into;

  fn into_zenoh(self) -> Self::Into;
}
