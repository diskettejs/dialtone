use napi_derive::napi;
use zenoh::handlers as zhandlers;
use zenoh::handlers::IntoHandler;

use crate::handlers::IntoZenoh;

#[derive(Clone, Default)]
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
}

impl IntoZenoh for FifoChannel {
  type Into = zhandlers::FifoChannel;

  fn into_zenoh(self) -> Self::Into {
    match self.capacity {
      Some(capacity) => zhandlers::FifoChannel::new(capacity),
      None => zhandlers::FifoChannel::default(),
    }
  }
}

impl<T: Send + 'static> IntoHandler<T> for FifoChannel {
  type Handler = zhandlers::FifoChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    self.into_zenoh().into_handler()
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

impl IntoZenoh for RingChannel {
  type Into = zhandlers::RingChannel;

  fn into_zenoh(self) -> Self::Into {
    zhandlers::RingChannel::new(self.capacity)
  }
}

impl<T: Send + 'static> IntoHandler<T> for RingChannel {
  type Handler = zhandlers::RingChannelHandler<T>;

  fn into_handler(self) -> (zhandlers::Callback<T>, Self::Handler) {
    self.into_zenoh().into_handler()
  }
}
