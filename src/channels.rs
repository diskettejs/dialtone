use napi_derive::napi;
use zenoh::handlers as zhandlers;
use zenoh::handlers::IntoHandler;

#[napi]
pub struct FifoChannel {
  inner: zhandlers::FifoChannel,
}

impl FifoChannel {
  pub fn with_capacity(capacity: Option<u32>) -> Self {
    capacity.map_or_else(Self::default, Self::new)
  }
}

#[napi]
impl FifoChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      inner: zhandlers::FifoChannel::new(capacity as usize),
    }
  }

  #[napi(factory)]
  pub fn default() -> Self {
    Self {
      inner: zhandlers::FifoChannel::default(),
    }
  }
}

#[napi]
pub struct RingChannel {
  inner: zhandlers::RingChannel,
}

impl<T: Send + 'static> IntoHandler<T> for RingChannel {
  type Handler = zhandlers::RingChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    self.inner.into_handler()
  }
}

#[napi]
impl RingChannel {
  #[napi(constructor)]
  pub fn new(capacity: u32) -> Self {
    Self {
      inner: zhandlers::RingChannel::new(capacity as usize),
    }
  }

  #[napi(factory)]
  pub fn default() -> Self {
    Self {
      inner: zhandlers::RingChannel::default(),
    }
  }
}

impl<T: Send + 'static> IntoHandler<T> for FifoChannel {
  type Handler = zhandlers::FifoChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    self.inner.into_handler()
  }
}
