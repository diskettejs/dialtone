use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use napi::Unknown;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh::handlers::{self as zhandlers, IntoHandler};

use crate::{macros::recv_handler, query::Reply, utils::MapNapiErr};

#[derive(Clone, Default)]
#[napi]
pub struct FifoChannel {
  capacity: usize,
}

#[napi]
impl FifoChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      capacity: capacity as usize,
    }
  }
}

impl<T: Send + 'static> IntoHandler<T> for FifoChannel {
  type Handler = zhandlers::FifoChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    zhandlers::FifoChannel::new(self.capacity).into_handler()
  }
}

#[derive(Clone)]
#[napi]
pub struct RingChannel {
  capacity: usize,
}

#[napi]
impl RingChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      capacity: capacity as usize,
    }
  }
}

impl<T: Send + 'static> IntoHandler<T> for RingChannel {
  type Handler = zhandlers::RingChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    zhandlers::RingChannel::new(self.capacity).into_handler()
  }
}

type RecvFuture<'a, T> = Pin<Box<dyn Future<Output = napi::Result<T>> + Send + 'a>>;

pub struct HandlerImpl<T>(Box<dyn Receiver<T>>);

impl<T> HandlerImpl<T> {
  pub(crate) fn recv(&self) -> RecvFuture<'_, T> {
    self.0.recv()
  }

  pub(crate) fn try_recv(&self) -> napi::Result<Option<T>> {
    self.0.try_recv()
  }
}

#[async_trait]
pub(crate) trait Receiver<T>: Send + Sync {
  async fn recv(&self) -> napi::Result<T>;
  fn try_recv(&self) -> napi::Result<Option<T>>;
}

#[napi]
pub struct ReplyHandler(HandlerImpl<zenoh::query::Reply>);

impl From<HandlerImpl<zenoh::query::Reply>> for ReplyHandler {
  fn from(value: HandlerImpl<zenoh::query::Reply>) -> Self {
    Self(value)
  }
}

recv_handler!(ReplyHandler.0 => Reply);

macro_rules! impl_receiver {
  ($($handler:ident),* $(,)?) => {$(
    #[async_trait]
    impl<T: Send + 'static> Receiver<T> for zhandlers::$handler<T> {
      async fn recv(&self) -> napi::Result<T> {
        self.recv_async().await.map_napi_err()
      }

      fn try_recv(&self) -> napi::Result<Option<T>> {
        Self::try_recv(self).map_napi_err()
      }
    }
  )*};
}
impl_receiver!(FifoChannelHandler, RingChannelHandler,);

fn erased<C, T>(channel: C) -> (zhandlers::Callback<T>, HandlerImpl<T>)
where
  C: IntoHandler<T>,
  C::Handler: Receiver<T> + 'static,
{
  let (callback, handler) = channel.into_handler();
  (callback, HandlerImpl(Box::new(handler)))
}

/// Owned, `Send`, `Env`-free channel resolved during argument conversion.
///
/// `from_napi_value` runs on the JS thread, where `Env` is available, so it narrows the JS
/// channel (`FifoChannel`/`RingChannel`) and erases it into the `(Callback, HandlerImpl)` pair
/// right there. Downstream holds only that pair — no `Env`, still `Send` — so a declaration can
/// run as a plain `async fn` without `spawn_future`.
pub struct ChannelHandler<T>(zhandlers::Callback<T>, HandlerImpl<T>);

impl<T: Send + 'static> FromNapiValue for ChannelHandler<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let obj = unsafe { Unknown::from_napi_value(env, napi_val)? };
    let (callback, handler) = match Either::<
      ClassInstance<'_, FifoChannel>,
      ClassInstance<'_, RingChannel>,
    >::from_unknown(obj)
    .map_err(|_| {
      napi::Error::from_reason("Invalid handler type: expected FifoChannel or RingChannel")
    })? {
      Either::A(h) => erased((*h).clone()),
      Either::B(h) => erased((*h).clone()),
    };
    Ok(Self(callback, handler))
  }
}

pub(crate) fn into_handler<T: Send + 'static>(
  handler: Option<ChannelHandler<T>>,
) -> impl IntoHandler<T, Handler = HandlerImpl<T>> {
  match handler {
    Some(ChannelHandler(callback, handler)) => (callback, handler),
    None => erased(zhandlers::FifoChannel::default()),
  }
}
