use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use napi::Unknown;
use napi::bindgen_prelude::*;
use zenoh::handlers::{self as zhandlers, IntoHandler};

use crate::{channels::*, error::MapNapiErr};

type RecvFuture<'a, T> = Pin<Box<dyn Future<Output = napi::Result<T>> + Send + 'a>>;

pub(crate) struct HandlerImpl<T>(Box<dyn Receiver<T>>);

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
impl_receiver!(FifoChannelHandler, RingChannelHandler);

fn erased<C, T>(channel: C) -> (zhandlers::Callback<T>, HandlerImpl<T>)
where
  C: IntoHandler<T>,
  C::Handler: Receiver<T> + 'static,
{
  let (callback, handler) = channel.into_handler();
  (callback, HandlerImpl(Box::new(handler)))
}

pub(crate) fn into_handler<'a, T>(
  handler: Option<Unknown<'a>>,
) -> napi::Result<impl IntoHandler<T, Handler = HandlerImpl<T>> + use<T>>
where
  T: Send + 'static,
{
  let Some(obj) = handler else {
    return Ok(erased(zhandlers::FifoChannel::default()));
  };

  let channel =
    Either::<ClassInstance<'_, FifoChannel>, ClassInstance<'_, RingChannel>>::from_unknown(obj)
      .map_err(|_| {
        napi::Error::from_reason("Invalid handler type: expected FifoChannel or RingChannel")
      })?;

  Ok(match channel {
    Either::A(h) => erased((*h).clone()),
    Either::B(h) => erased((*h).clone()),
  })
}
