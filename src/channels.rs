use std::time::{Duration, Instant};

use napi::bindgen_prelude::*;
use napi::futures_core::Stream;
use napi::tokio_stream::StreamExt;
use napi_derive::napi;
use zenoh::handlers as zhandlers;

use crate::error::*;
use crate::miss::Miss;
use crate::query::Query;
use crate::sample::Sample;

#[napi]
pub struct FifoChannel {
  capacity: Option<usize>,
}

#[napi]
impl FifoChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      capacity: Some(capacity as usize),
    }
  }

  #[napi(factory)]
  pub fn default() -> Self {
    Self { capacity: None }
  }
}

#[napi]
pub struct RingChannel {
  capacity: Option<usize>,
}

#[napi]
impl RingChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      capacity: Some(capacity as usize),
    }
  }

  #[napi(factory)]
  pub fn default() -> Self {
    Self { capacity: None }
  }
}

// TODO: add all possible channel handler payloads
#[napi]
pub type HandlerPayload = Either3<Sample, Query, Miss>;

/// Bridges a Zenoh handler payload to the JavaScript value produced for it.
///
/// `Source` is the type Zenoh delivers through the channel; `Self` is the napi
/// class the channel handlers hand back to JavaScript.
pub trait ChannelPayload: ToNapiValue + 'static {
  type Source: Send + 'static;

  fn from_source(source: Self::Source) -> Self;
}

impl ChannelPayload for Sample {
  type Source = zenoh::sample::Sample;

  fn from_source(source: zenoh::sample::Sample) -> Self {
    Sample::new(source)
  }
}

impl ChannelPayload for Query {
  type Source = zenoh::query::Query;

  fn from_source(source: zenoh::query::Query) -> Self {
    source.into()
  }
}

pub struct DialtoneFifoHandler<P: ChannelPayload> {
  inner: zhandlers::FifoChannelHandler<P::Source>,
}

impl<P: ChannelPayload> DialtoneFifoHandler<P> {
  pub fn new(inner: zhandlers::FifoChannelHandler<P::Source>) -> Self {
    Self { inner }
  }

  pub fn try_recv(&self) -> Result<Option<P>> {
    Ok(self.inner.try_recv().map_napi_err()?.map(P::from_source))
  }

  pub fn recv(&self) -> Result<P> {
    self.inner.recv().map_napi_err().map(P::from_source)
  }

  pub async fn recv_async(&self) -> Result<P> {
    self
      .inner
      .recv_async()
      .await
      .map_napi_err()
      .map(P::from_source)
  }

  // TODO: revisit blocking behavior — these proxy Zenoh's synchronous recv_deadline/
  // recv_timeout, which park the calling thread until an item is ready or time elapses.
  pub fn recv_deadline(&self, deadline: Instant) -> Result<Option<P>> {
    Ok(
      self
        .inner
        .recv_deadline(deadline)
        .map_napi_err()?
        .map(P::from_source),
    )
  }

  pub fn recv_timeout(&self, timeout: Duration) -> Result<Option<P>> {
    Ok(
      self
        .inner
        .recv_timeout(timeout)
        .map_napi_err()?
        .map(P::from_source),
    )
  }

  pub fn try_iter(&self) -> Vec<P> {
    self.inner.try_iter().map(P::from_source).collect()
  }

  pub fn drain(&self) -> Vec<P> {
    self.inner.drain().map(P::from_source).collect()
  }

  pub fn stream(&self) -> impl Stream<Item = P> + '_ {
    self.inner.stream().map(P::from_source)
  }

  pub fn is_disconnected(&self) -> bool {
    self.inner.is_disconnected()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  pub fn is_full(&self) -> bool {
    self.inner.is_full()
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn capacity(&self) -> Option<usize> {
    self.inner.capacity()
  }

  pub fn sender_count(&self) -> usize {
    self.inner.sender_count()
  }

  pub fn receiver_count(&self) -> usize {
    self.inner.receiver_count()
  }

  pub fn same_channel(&self, other: &Self) -> bool {
    self.inner.same_channel(&other.inner)
  }
}

pub struct DialtoneRingHandler<P: ChannelPayload> {
  inner: zhandlers::RingChannelHandler<P::Source>,
}

impl<P: ChannelPayload> DialtoneRingHandler<P> {
  pub fn new(inner: zhandlers::RingChannelHandler<P::Source>) -> Self {
    Self { inner }
  }

  pub fn try_recv(&self) -> Result<Option<P>> {
    Ok(self.inner.try_recv().map_napi_err()?.map(P::from_source))
  }

  pub fn recv(&self) -> Result<P> {
    self.inner.recv().map_napi_err().map(P::from_source)
  }

  pub async fn recv_async(&self) -> Result<P> {
    self
      .inner
      .recv_async()
      .await
      .map_napi_err()
      .map(P::from_source)
  }

  // TODO: revisit blocking behavior — these proxy Zenoh's synchronous recv_deadline/
  // recv_timeout, which park the calling thread until an item is ready or time elapses.
  pub fn recv_deadline(&self, deadline: Instant) -> Result<Option<P>> {
    Ok(
      self
        .inner
        .recv_deadline(deadline)
        .map_napi_err()?
        .map(P::from_source),
    )
  }

  pub fn recv_timeout(&self, timeout: Duration) -> Result<Option<P>> {
    Ok(
      self
        .inner
        .recv_timeout(timeout)
        .map_napi_err()?
        .map(P::from_source),
    )
  }
}
