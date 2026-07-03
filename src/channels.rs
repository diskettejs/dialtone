use napi_derive::napi;
use zenoh::handlers as zhandlers;
use zenoh::handlers::IntoHandler;

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
