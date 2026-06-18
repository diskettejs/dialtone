//! Internal macros shared across the bindings.

/// Apply the QoS and addressing options shared by `put`, `delete`, and
/// `declare_publisher` onto a Zenoh builder, returning the updated builder.
///
/// These three methods build three distinct builder types that nonetheless
/// expose the same `congestion_control` / `priority` / `express` /
/// `allowed_destination` setters. A macro shares the option-applying logic by
/// duck typing over the builder type; a generic fn would need Zenoh's
/// `QoSBuilderTrait` as a bound, which isn't reachable without enabling Zenoh's
/// `internal` feature. `$options` is consumed field-by-field, so any non-shared
/// options (`encoding`, `attachment`) remain available to the caller afterwards.
macro_rules! apply_common_options {
  ($builder:expr, $options:ident) => {{
    let mut builder = $builder;
    if let Some(congestion_control) = $options.congestion_control {
      builder = builder.congestion_control(congestion_control.into());
    }
    if let Some(priority) = $options.priority {
      builder = builder.priority(priority.into());
    }
    if let Some(express) = $options.express {
      builder = builder.express(express);
    }
    if let Some(allowed_destination) = $options.allowed_destination {
      builder = builder.allowed_destination(allowed_destination.into());
    }
    builder
  }};
}

pub(crate) use apply_common_options;
