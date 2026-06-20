use napi_derive::napi;

/// Whether a publisher currently has matching subscribers (or, later, a querier
/// has matching queryables).
#[napi(object)]
pub struct MatchingStatus {
  /// `true` if at least one matching entity currently exists.
  pub matching: bool,
}
