//! `Liveliness` — the liveliness sub-API of a `Session`, reached via
//! `Session.liveliness()`.
//!
//! zenoh's `Liveliness<'a>` borrows the session, so it can't be stored as-is.
//! Like the other entry points, the wrapper holds an owned `zenoh::Session`
//! clone (cheap — it's `Arc`-backed) and spins up a fresh `Liveliness<'_>` per
//! call.

use napi_derive::napi;
use zenoh::handlers::{FifoChannel, RingChannel};

use crate::handlers::{ChannelKind, DEFAULT_CHANNEL_CAPACITY};
use crate::keyexpr::KeyExprArg;
use crate::liveliness_subscriber::LivelinessSubscriber;
use crate::liveliness_token::LivelinessToken;
use crate::options::LivelinessSubscriberOptions;

#[napi]
pub struct Liveliness {
  session: zenoh::Session,
}

impl Liveliness {
  /// Internal constructor: hold an owned session clone to mint `Liveliness<'_>`.
  pub(crate) fn from_session(session: zenoh::Session) -> Self {
    Liveliness { session }
  }
}

#[napi]
impl Liveliness {
  /// Declares a liveliness token on `keyExpr`. The token asserts this session's
  /// liveliness for that key expression until it is undeclared or dropped.
  #[napi]
  pub async fn declare_token(
    &self,
    #[napi(ts_arg_type = "string | KeyExpr")] key_expr: KeyExprArg,
  ) -> napi::Result<LivelinessToken> {
    let liveliness = self.session.liveliness();
    let token = liveliness
      .declare_token(key_expr.0)
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(LivelinessToken::from_inner(token))
  }

  /// Declares a subscription to liveliness changes matching `keyExpr`.
  ///
  /// The `handler` option chooses the channel (default: FIFO of
  /// [`DEFAULT_CHANNEL_CAPACITY`]); `history` replays the current matching
  /// tokens on declaration.
  #[napi]
  pub async fn declare_subscriber(
    &self,
    #[napi(ts_arg_type = "string | KeyExpr")] key_expr: KeyExprArg,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let handler_cfg = options.as_ref().and_then(|o| o.handler.as_ref());
    let capacity = handler_cfg
      .and_then(|c| c.capacity)
      .map(|c| c as usize)
      .unwrap_or(DEFAULT_CHANNEL_CAPACITY);
    let is_ring = handler_cfg.is_some_and(|c| matches!(c.kind, ChannelKind::Ring));
    let history = options.as_ref().and_then(|o| o.history).unwrap_or(false);

    let liveliness = self.session.liveliness();
    let mut builder = liveliness.declare_subscriber(key_expr.0);
    if history {
      builder = builder.history(true);
    }

    if is_ring {
      let sub = builder
        .with(RingChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      let key_expr = sub.key_expr().clone();
      let id = sub.id();
      Ok(LivelinessSubscriber::from_ring(sub, key_expr, id))
    } else {
      let sub = builder
        .with(FifoChannel::new(capacity))
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
      let key_expr = sub.key_expr().clone();
      let id = sub.id();
      Ok(LivelinessSubscriber::from_fifo(sub, key_expr, id))
    }
  }
}
