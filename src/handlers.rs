use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use napi::Unknown;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use zenoh as z;

use crate::utils::MapNapiErr;

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

impl<T: Send + 'static> z::handlers::IntoHandler<T> for FifoChannel {
  type Handler = z::handlers::FifoChannelHandler<T>;

  fn into_handler(self) -> (z::handlers::Callback<T>, Self::Handler) {
    z::handlers::FifoChannel::new(self.capacity).into_handler()
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

impl<T: Send + 'static> z::handlers::IntoHandler<T> for RingChannel {
  type Handler = z::handlers::RingChannelHandler<T>;

  fn into_handler(self) -> (z::handlers::Callback<T>, Self::Handler) {
    z::handlers::RingChannel::new(self.capacity).into_handler()
  }
}

/// Maps a raw Zenoh payload type to its napi wrapper class, converted only at the JS boundary.
pub(crate) trait IntoJs: Sized + Send + Sync + 'static {
  type Into: ToNapiValue;

  fn into_js(self) -> Self::Into;
}

macro_rules! into_js {
  ($($raw:ty => $wrapper:ty),* $(,)?) => {$(
    impl IntoJs for $raw {
      type Into = $wrapper;

      fn into_js(self) -> Self::Into {
        self.into()
      }
    }
  )*};
}
into_js! {
  zenoh::sample::Sample => crate::sample::Sample,
  zenoh::query::Reply => crate::query::Reply,
  zenoh::query::Query => crate::query::Query,
  zenoh::scouting::Hello => crate::scout::Hello,
  zenoh::matching::MatchingStatus => crate::matching::MatchingStatus,
  zenoh_ext::Miss => crate::miss::Miss,
  zenoh::session::TransportEvent => crate::session::TransportEvent,
  zenoh::session::LinkEvent => crate::session::LinkEvent,
}

/// A received payload, type-erased, carrying its own deferred wrapper conversion.
/// napi invokes `ToNapiValue` on the JS thread — at promise resolution for async
/// returns, immediately for sync returns — so no `Env` is needed anywhere upstream.
/// Appears as `DeferredJs` in the generated d.ts (declared `unknown` via dts header);
/// payload typing lives in the handwritten TS facade.
pub struct DeferredJs(Box<dyn FnOnce(sys::napi_env) -> Result<sys::napi_value> + Send>);

impl DeferredJs {
  fn new<T: IntoJs>(value: T) -> Self {
    Self(Box::new(move |env| unsafe {
      <T::Into as ToNapiValue>::to_napi_value(env, value.into_js())
    }))
  }
}

impl ToNapiValue for DeferredJs {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    (val.0)(env)
  }
}

#[async_trait]
pub(crate) trait Receiver: Send + Sync {
  async fn recv(&self) -> napi::Result<DeferredJs>;
  fn try_recv(&self) -> napi::Result<Option<DeferredJs>>;
}

macro_rules! impl_receiver {
  ($($handler:ident),* $(,)?) => {$(
    #[async_trait]
    impl<T: IntoJs> Receiver for z::handlers::$handler<T> {
      async fn recv(&self) -> napi::Result<DeferredJs> {
        Ok(DeferredJs::new(self.recv_async().await.map_napi_err()?))
      }

      fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
        Ok(Self::try_recv(self).map_napi_err()?.map(DeferredJs::new))
      }
    }
  )*};
}
impl_receiver!(FifoChannelHandler, RingChannelHandler);

#[derive(Clone)]
#[napi]
pub struct Handler(Arc<dyn Receiver>);

#[napi]
impl Handler {
  #[napi]
  pub async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.recv().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.try_recv()
  }

  #[napi]
  pub fn stream(&self) -> Stream {
    Stream(self.clone())
  }
}

#[napi(async_iterator)]
pub struct Stream(Handler);

#[napi]
impl AsyncGenerator for Stream {
  type Yield = DeferredJs;
  type Next = ();
  type Return = ();

  fn next(
    &mut self,
    _value: Option<()>,
  ) -> impl std::future::Future<Output = napi::Result<Option<DeferredJs>>> + Send + 'static {
    let handler = self.0.clone();
    async move { Ok(handler.recv().await.ok()) }
  }
}

pub struct HandlerImpl<T>(Handler, PhantomData<T>);

impl<T> Clone for HandlerImpl<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone(), PhantomData)
  }
}

impl<T> HandlerImpl<T> {
  pub(crate) async fn recv(&self) -> napi::Result<DeferredJs> {
    self.0.recv().await
  }

  pub(crate) fn try_recv(&self) -> napi::Result<Option<DeferredJs>> {
    self.0.try_recv()
  }

  pub(crate) fn stream(&self) -> Stream {
    self.0.stream()
  }
}

impl<T> From<HandlerImpl<T>> for Handler {
  fn from(value: HandlerImpl<T>) -> Self {
    value.0
  }
}

fn erased<C, T>(channel: C) -> (z::handlers::Callback<T>, HandlerImpl<T>)
where
  C: z::handlers::IntoHandler<T>,
  C::Handler: Receiver + 'static,
{
  let (callback, handler) = channel.into_handler();
  (
    callback,
    HandlerImpl(Handler(Arc::new(handler)), PhantomData),
  )
}

/// Owned, `Send`, `Env`-free channel resolved during argument conversion.
///
/// `from_napi_value` runs on the JS thread, where `Env` is available, so it narrows the JS
/// channel (`FifoChannel`/`RingChannel`) and erases it into the `(Callback, HandlerImpl)` pair
/// right there. Downstream holds only that pair — no `Env`, still `Send` — so a declaration can
/// run as a plain `async fn` without `spawn_future`.
pub struct ChannelHandler<T>(z::handlers::Callback<T>, HandlerImpl<T>);

impl<T: IntoJs> FromNapiValue for ChannelHandler<T> {
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

pub(crate) fn into_handler<T: IntoJs>(
  handler: Option<ChannelHandler<T>>,
) -> impl z::handlers::IntoHandler<T, Handler = HandlerImpl<T>> {
  match handler {
    Some(ChannelHandler(callback, handler)) => (callback, handler),
    None => erased(z::handlers::FifoChannel::default()),
  }
}
