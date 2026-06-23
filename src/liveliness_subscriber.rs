use std::sync::Arc;

use napi::bindgen_prelude::Either;
use napi_derive::napi;
use zenoh::handlers::{FifoChannelHandler, RingChannelHandler};
use zenoh::key_expr::KeyExpr as ZKeyExpr;
use zenoh::pubsub::Subscriber as ZSubscriber;
use zenoh::sample::Sample as ZSample;
use zenoh::session::EntityGlobalId as ZEntityGlobalId;

use crate::entity_global_id::EntityGlobalId;
use crate::handlers::{FifoChannelHandlerSample, RingChannelHandlerSample};
use crate::keyexpr::KeyExpr;

// Unlike `Subscriber` (which wraps `zenoh_ext::AdvancedSubscriber`), a liveliness
// subscription is a plain `zenoh::Subscriber` — liveliness has no advanced
// variant. It still yields `Sample`s (a `Put` marks a token appearing, a `Delete`
// a token disappearing), so it reuses the `Sample` channel-handler classes.
enum SubInner {
  Fifo(ZSubscriber<FifoChannelHandler<ZSample>>),
  Ring(Arc<ZSubscriber<RingChannelHandler<ZSample>>>),
}

/// A subscription to liveliness changes on a key expression, declared via
/// `Liveliness.declareSubscriber`.
#[napi]
pub struct LivelinessSubscriber {
  // `None` once undeclared. `key_expr`/`id` are cached so they survive it.
  inner: Option<SubInner>,
  key_expr: ZKeyExpr<'static>,
  id: ZEntityGlobalId,
}

impl LivelinessSubscriber {
  pub(crate) fn from_fifo(
    sub: ZSubscriber<FifoChannelHandler<ZSample>>,
    key_expr: ZKeyExpr<'static>,
    id: ZEntityGlobalId,
  ) -> Self {
    LivelinessSubscriber {
      inner: Some(SubInner::Fifo(sub)),
      key_expr,
      id,
    }
  }

  pub(crate) fn from_ring(
    sub: ZSubscriber<RingChannelHandler<ZSample>>,
    key_expr: ZKeyExpr<'static>,
    id: ZEntityGlobalId,
  ) -> Self {
    LivelinessSubscriber {
      inner: Some(SubInner::Ring(Arc::new(sub))),
      key_expr,
      id,
    }
  }
}

#[napi]
impl LivelinessSubscriber {
  /// The key expression this subscription matches.
  #[napi(getter)]
  pub fn key_expr(&self) -> KeyExpr {
    KeyExpr::from_inner(self.key_expr.clone())
  }

  /// The global id of this subscription entity.
  #[napi(getter)]
  pub fn id(&self) -> EntityGlobalId {
    EntityGlobalId::from_inner(self.id)
  }

  /// The receive end of the subscription. A `FifoChannelHandler` or
  /// `RingChannelHandler` depending on the channel chosen at declare time.
  ///
  /// The handler is not iterable; iterate via `subscriber.handler.stream()`.
  #[napi(getter)]
  pub fn handler(
    &self,
  ) -> napi::Result<Either<FifoChannelHandlerSample, RingChannelHandlerSample>> {
    match self.inner.as_ref() {
      Some(SubInner::Fifo(sub)) => Ok(Either::A(FifoChannelHandlerSample::from_handler(
        sub.handler().clone(),
      ))),
      Some(SubInner::Ring(arc)) => Ok(Either::B(RingChannelHandlerSample::from_arc(Arc::clone(
        arc,
      )))),
      None => Err(napi::Error::from_reason(
        "liveliness subscriber has been undeclared",
      )),
    }
  }

  /// Undeclare this subscription. Resolves once undeclaration completes; a
  /// second call is a no-op.
  ///
  /// For a ring subscription still referenced by an outstanding handler, this
  /// drops our strong reference and lets the background drop undeclare it once
  /// the last handler is released.
  #[napi]
  pub async unsafe fn undeclare(&mut self) -> napi::Result<()> {
    match self.inner.take() {
      Some(SubInner::Fifo(sub)) => sub
        .undeclare()
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string())),
      Some(SubInner::Ring(arc)) => match Arc::try_unwrap(arc) {
        Ok(sub) => sub
          .undeclare()
          .await
          .map_err(|e| napi::Error::from_reason(e.to_string())),
        Err(_) => Ok(()),
      },
      None => Ok(()),
    }
  }
}
