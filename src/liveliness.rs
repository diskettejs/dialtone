use derive_more::From;
use napi_derive::napi;
use zenoh as z;

use crate::{
  config::EntityGlobalId,
  key_expr::{KeyExpr, KeyExprArg},
  options::{LivelinessGetOptions, LivelinessSubscriberOptions},
  pubsub::SampleIter,
  query::Replies,
  utils::{Declared, MapNapiErr, duration_ms, fifo},
};

/// The liveliness interface of a session: declares liveliness tokens, queries the ones
/// currently alive and subscribes to their changes.
///
/// A liveliness token is a token whose liveliness is tied to the session that declared it
/// and that can be monitored by remote applications.
#[napi]
#[derive(From)]
pub struct Liveliness(z::Session);

#[napi]
impl Liveliness {
  /// Declares a liveliness token on the given key expression.
  ///
  /// The token is seen as alive by the applications monitoring it for as long as it is
  /// not undeclared, the declaring application is running and the two are connected.
  #[napi]
  pub async fn declare_token(&self, key_expr: KeyExprArg<'_>) -> napi::Result<LivelinessToken> {
    let expr = KeyExpr::try_from(key_expr)?;
    let token = self
      .0
      .liveliness()
      .declare_token(expr)
      .await
      .map_napi_err()?;

    Ok(token.into())
  }

  /// Declares a subscriber for the liveliness changes of the tokens matching the given
  /// key expression.
  ///
  /// A sample whose kind is `Put` signals a token that became alive; one whose kind is
  /// `Delete` signals a token that was lost.
  #[napi]
  pub async fn declare_subscriber(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<LivelinessSubscriberOptions>,
  ) -> napi::Result<LivelinessSubscriber> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessSubscriberOptions {
      history,
      channel_capacity,
    } = options.unwrap_or_default();
    let handler = fifo(channel_capacity);

    let subscriber = self
      .0
      .liveliness()
      .declare_subscriber(expr)
      .with(handler)
      .history(history.unwrap_or(false))
      .await
      .map_napi_err()?;

    Ok(subscriber.into())
  }

  /// Queries the liveliness tokens currently alive whose key expression matches the given
  /// one.
  ///
  /// @returns The replies to the query, each carrying the key expression of one live
  /// token.
  #[napi]
  pub async fn get(
    &self,
    key_expr: KeyExprArg<'_>,
    options: Option<LivelinessGetOptions>,
  ) -> napi::Result<Replies> {
    let expr = KeyExpr::try_from(key_expr)?;
    let LivelinessGetOptions {
      timeout,
      // cancellation_token,
      channel_capacity,
    } = options.unwrap_or_default();

    let timeout = duration_ms(timeout)?;
    let handler = fifo(channel_capacity);
    let session = self.0.clone();
    let mut builder = session.liveliness().get(expr).with(handler);

    if let Some(timeout) = timeout {
      builder = builder.timeout(timeout);
    }

    let rec = builder.await.map_napi_err()?;

    Ok(rec.into())
  }
}

/// A token whose liveliness is tied to the session that declared it.
///
/// The token is seen as alive by any application monitoring it for as long as the token
/// is not undeclared, the declaring application is running and the two applications have
/// Zenoh connectivity.
///
/// {@link LivelinessToken.undeclare} consumes the token; it throws once the token has been
/// undeclared.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct LivelinessToken(Declared<z::liveliness::LivelinessToken>);

#[napi]
impl LivelinessToken {
  /// Undeclares this token, so that the applications monitoring it stop seeing it as
  /// alive.
  ///
  /// @throws If this token has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }
}

/// A subscriber receiving the liveliness changes of the tokens matching its key
/// expression.
///
/// {@link LivelinessSubscriber.undeclare} consumes the subscriber; every member throws
/// once it has been undeclared.
#[napi]
#[derive(From)]
#[from(forward)]
pub struct LivelinessSubscriber(
  Declared<z::pubsub::Subscriber<z::handlers::FifoChannelHandler<z::sample::Sample>>>,
);

#[napi]
impl LivelinessSubscriber {
  /// The key expression whose liveliness changes this subscriber receives.
  #[napi(getter)]
  pub fn key_expr(&self) -> napi::Result<KeyExpr> {
    Ok(self.0.get()?.key_expr().clone().into())
  }

  /// The global identifier of this subscriber.
  #[napi(getter)]
  pub fn id(&self) -> napi::Result<EntityGlobalId> {
    Ok(self.0.get()?.id().into())
  }

  /// Undeclares this subscriber, so that no further liveliness change is delivered to it.
  ///
  /// @throws If this subscriber has already been undeclared.
  #[napi]
  pub fn undeclare(&mut self) -> napi::Result<()> {
    z::Wait::wait(self.0.take()?.undeclare()).map_napi_err()
  }

  /// Iterates over the liveliness changes delivered to this subscriber.
  ///
  /// @returns A {@link `SampleIter`} that yields a sample whose kind is `Put` for each
  /// token that became alive and `Delete` for each token that was lost, and completes
  /// once the subscriber stops receiving.
  /// @throws If this subscriber has already been undeclared.
  #[napi]
  pub fn receive(&self) -> napi::Result<SampleIter> {
    let handler = self.0.get()?.handler().clone();

    Ok(handler.into())
  }
}
