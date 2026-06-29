use napi::{Env, bindgen_prelude::*};
use napi_derive::napi;
use zenoh::{
  handlers::{self as zhandlers, IntoHandler},
  key_expr as zkey_expr, liveliness as zliveliness, sample as zsample, session as zsession,
};

use crate::{
  channels::*, config::*, error::*, key_expr::*, liveliness::*, miss::*, options::*, sample::Sample,
};

#[napi]
pub struct Subscriber {
  id: zsession::EntityGlobalId,
  key_expr: zkey_expr::KeyExpr<'static>,
  inner: Option<zenoh_ext::AdvancedSubscriber<()>>,
  receiver: crate::handlers::FifoChannelHandler<zsample::Sample>,
}

impl Subscriber {
  pub(crate) fn new(
    inner: zenoh_ext::AdvancedSubscriber<()>,
    receiver: zhandlers::FifoChannelHandler<zsample::Sample>,
  ) -> Self {
    Self {
      id: inner.id(),
      key_expr: inner.key_expr().clone(),
      inner: Some(inner),
      receiver: receiver.into(),
    }
  }

  fn get(&self) -> napi::Result<&zenoh_ext::AdvancedSubscriber<()>> {
    self
      .inner
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("subscriber has already been undeclared"))
  }
}

#[napi]
impl Subscriber {
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    self.key_expr.clone().into()
  }

  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    self.id.into()
  }

  #[napi]
  pub async fn recv(&self) -> napi::Result<Sample> {
    self.receiver.recv::<Sample>().await
  }

  #[napi]
  pub fn try_recv(&self) -> napi::Result<Option<Sample>> {
    self.receiver.try_recv::<Sample>()
  }

  #[napi]
  pub fn drain(&self) -> Vec<Sample> {
    self.receiver.drain::<Sample>()
  }

  #[napi]
  pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, Sample>> {
    self.receiver.stream::<Sample>(env)
  }

  #[napi]
  pub fn is_empty(&self) -> bool {
    self.receiver.is_empty()
  }

  #[napi]
  pub fn is_full(&self) -> bool {
    self.receiver.is_full()
  }

  #[napi]
  pub fn is_disconnected(&self) -> bool {
    self.receiver.is_disconnected()
  }

  #[napi]
  pub fn len(&self) -> u32 {
    self.receiver.len()
  }

  #[napi]
  pub fn capacity(&self) -> Option<u32> {
    self.receiver.capacity()
  }

  #[napi]
  pub fn sender_count(&self) -> u32 {
    self.receiver.sender_count()
  }

  #[napi]
  pub fn receiver_count(&self) -> u32 {
    self.receiver.receiver_count()
  }

  #[napi]
  pub async fn sample_miss_listener(
    &self,
    options: Option<SampleMissListenerOptions>,
  ) -> napi::Result<SampleMissListener> {
    let SampleMissListenerOptions { capacity } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();

    let sample_listener = self
      .get()?
      .sample_miss_listener()
      .with((cb, ()))
      .await
      .map_napi_err()?;

    Ok(SampleMissListener::new(sample_listener, receiver))
  }

  #[napi]
  pub async fn detect_publishers(
    &self,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let LivelinessSubscriberOptions { history, capacity } = options.unwrap_or_default();
    let (cb, receiver) = FifoChannel::with_capacity(capacity).into_handler();
    let mut builder = self.get()?.detect_publishers().with((cb, ()));

    if let Some(history) = history {
      builder = builder.history(history);
    }

    let subscriber = builder.await.map_napi_err()?;

    Ok(LivelinessSubscriber::new(subscriber, receiver))
  }

  #[napi]
  pub fn undeclare<'env>(&mut self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let subscriber = self
      .inner
      .take()
      .ok_or_else(|| napi::Error::from_reason("subscriber has already been undeclared"))?;

    env.spawn_future(async move { subscriber.undeclare().await.map_napi_err() })
  }
}
