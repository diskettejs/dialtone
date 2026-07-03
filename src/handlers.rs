use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use napi::Unknown;
use napi::bindgen_prelude::*;
use zenoh::handlers::{self as zhandlers, IntoHandler};

use crate::channels::{FifoChannel, RingChannel};
use crate::error::MapNapiErr;

type RustCallback<T> = zhandlers::Callback<T>;
type RecvFuture<'a, T> = Pin<Box<dyn Future<Output = napi::Result<T>> + Send + 'a>>;

pub(crate) enum HandlerImpl<T> {
  Rust(Box<dyn Receiver<T> + Send + Sync>),
  // JavaScript(CustomObject)
}

impl<T> HandlerImpl<T> {
  pub(crate) fn recv(&self) -> RecvFuture<'_, T> {
    match self {
      Self::Rust(receiver) => receiver.recv(),
    }
  }

  pub(crate) fn try_recv(&self) -> napi::Result<Option<T>> {
    match self {
      Self::Rust(receiver) => receiver.try_recv(),
    }
  }
}

pub(crate) trait IntoZenoh: 'static {
  type Into;

  fn into_zenoh(self) -> Self::Into;
}

pub(crate) trait Receiver<T>: Send + Sync {
  fn recv(&self) -> RecvFuture<'_, T>;
  fn try_recv(&self) -> napi::Result<Option<T>>;
}

struct RustHandler<H, T>
where
  H: IntoZenoh,
  H::Into: IntoHandler<T>,
{
  handler: <H::Into as IntoHandler<T>>::Handler,
  _phantom: PhantomData<T>,
}

macro_rules! impl_receiver {
  ($($channel:ident),* $(,)?) => {$(
    impl<T: Send + Sync + 'static> Receiver<T> for RustHandler<$channel, T> {
      fn recv(&self) -> RecvFuture<'_, T> {
        Box::pin(async move { self.handler.recv_async().await.map_napi_err() })
      }

      fn try_recv(&self) -> napi::Result<Option<T>> {
        self.handler.try_recv().map_napi_err()
      }
    }
  )*};
}
impl_receiver!(FifoChannel, RingChannel);

fn rust_handler<H, T>(channel: H) -> (RustCallback<T>, HandlerImpl<T>)
where
  H: IntoZenoh,
  H::Into: IntoHandler<T>,
  <H::Into as IntoHandler<T>>::Handler: Send + Sync + 'static,
  RustHandler<H, T>: Receiver<T>,
  T: Send + Sync + 'static,
{
  let (callback, handler) = channel.into_zenoh().into_handler();
  let rust_handler = RustHandler::<H, T> {
    handler,
    _phantom: PhantomData,
  };
  (callback, HandlerImpl::Rust(Box::new(rust_handler)))
}

pub(crate) fn into_handler<'a, T>(
  handler: Option<Unknown<'a>>,
) -> napi::Result<impl IntoHandler<T, Handler = HandlerImpl<T>> + use<T>>
where
  T: Send + Sync + 'static,
{
  let Some(obj) = handler else {
    return Ok(rust_handler::<FifoChannel, T>(FifoChannel::default()));
  };

  let channel = Either::<
    ClassInstance<'_, FifoChannel>,
    ClassInstance<'_, RingChannel>,
  >::from_unknown(obj)
  .map_err(|_| napi::Error::from_reason("Invalid handler type"))?;

  Ok(match channel {
    Either::A(h) => rust_handler::<FifoChannel, T>((*h).clone()),
    Either::B(h) => rust_handler::<RingChannel, T>((*h).clone()),
  })
}
