/// Generates `#[napi] pub struct $Name { inner: $Wrapped }` plus all 3 `From` conversions:
///   impl From<$Wrapped> for $Name  (move)
///   impl From<&$Name> for $Wrapped  (clones $Wrapped)
///   impl From<$Name> for $Wrapped  (move)
/// Struct name defaults to the wrapped path's last segment (peeled recursively) or an
/// explicit `as Name` override.
///
/// wrapper!(zkey_expr::KeyExpr<'static>: Clone);
/// wrapper!(zconfig::Config);
/// wrapper!(zbytes::ZBytes as Bytes);
macro_rules! wrapper {
    (
        $($seg:ident)::+ $(<$lt:lifetime>)?
        $(as $Name:ident)?
        $(: $($derive:path),+ $(,)?)?
    ) => {
        $crate::macros::wrapper!(@peel
            $($seg)::+ ;
            $($seg)::+ $(<$lt>)? ;
            $(as $Name)? ;
            $(derive($($derive),+))? ;
        );
    };
    (@peel
        $head:ident :: $($tail:ident)::+ ;
        $full:path ;
        $(as $Name:ident)? ;
        $(derive($($derive:path),+))? ;
    ) => {
        $crate::macros::wrapper!(@peel
            $($tail)::+ ; $full ; $(as $Name)? ; $(derive($($derive),+))? ;
        );
    };
    (@peel $last:ident ; $full:path ; ; $(derive($($derive:path),+))? ;) => {
        $crate::macros::wrapper!(@emit $last, $full ; $(derive($($derive),+))?);
    };
    (@peel $last:ident ; $full:path ; as $Name:ident ; $(derive($($derive:path),+))? ;) => {
        $crate::macros::wrapper!(@emit $Name, $full ; $(derive($($derive),+))?);
    };
    (@emit $Name:ident, $full:path ; $(derive($($derive:path),+))?) => {
        $(#[derive($($derive),+)])?
        #[napi]
        pub struct $Name {
            inner: $full,
        }

        impl From<$full> for $Name {
            fn from(inner: $full) -> Self {
                Self { inner }
            }
        }

        #[allow(clippy::clone_on_copy)]
        impl From<&$Name> for $full {
            fn from(value: &$Name) -> Self {
                value.inner.clone()
            }
        }

        impl From<$Name> for $full {
            fn from(value: $Name) -> Self {
                value.inner
            }
        }
    };
}
pub(crate) use wrapper;

/// Ported from zenoh-python's `option_wrapper!` (src/macros.rs). Wraps a Zenoh entity that
/// can be undeclared/taken exactly once; access after that returns `$error`. Adapted:
/// `napi::Error` instead of `PyErr`, no GIL release in `Drop` since napi has no GIL.
///
/// option_wrapper!(zenoh_ext::AdvancedSubscriber<HandlerImpl<Sample>> as Subscriber, "subscriber already undeclared");
/// option_wrapper!(zenoh::pubsub::Publisher<'static>, "publisher already undeclared");
macro_rules! option_wrapper {
    (
        $($seg:ident)::+ $(<$lt:lifetime>)?
        $(as $Name:ident)?
        , $error:literal $(,)?
    ) => {
        $crate::macros::option_wrapper!(@peel
            $($seg)::+ ;
            $($seg)::+ $(<$lt>)? ;
            $(as $Name)? ;
            $error ;
        );
    };
    (
        $($seg:ident)::+ $(<$($arg:ty),+>)?
        $(as $Name:ident)?
        , $error:literal $(,)?
    ) => {
        $crate::macros::option_wrapper!(@peel
            $($seg)::+ ;
            $($seg)::+ $(<$($arg),+>)? ;
            $(as $Name)? ;
            $error ;
        );
    };
    (@peel
        $head:ident :: $($tail:ident)::+ ;
        $full:path ;
        $(as $Name:ident)? ;
        $error:literal ;
    ) => {
        $crate::macros::option_wrapper!(@peel
            $($tail)::+ ; $full ; $(as $Name)? ; $error ;
        );
    };
    (@peel $last:ident ; $full:path ; ; $error:literal ;) => {
        $crate::macros::option_wrapper!(@emit $last, $full, $error);
    };
    (@peel $last:ident ; $full:path ; as $Name:ident ; $error:literal ;) => {
        $crate::macros::option_wrapper!(@emit $Name, $full, $error);
    };
    (@emit $Name:ident, $full:path, $error:literal) => {
        #[napi]
        pub struct $Name {
            inner: Option<$full>,
        }

        #[allow(dead_code)]
        impl $Name {
            fn none() -> napi::Error {
                napi::Error::from_reason($error)
            }

            fn get_ref(&self) -> napi::Result<&$full> {
                self.inner.as_ref().ok_or_else(Self::none)
            }

            fn get_mut(&mut self) -> napi::Result<&mut $full> {
                self.inner.as_mut().ok_or_else(Self::none)
            }

            fn take(&mut self) -> napi::Result<$full> {
                self.inner.take().ok_or_else(Self::none)
            }
        }

        impl From<$full> for $Name {
            fn from(value: $full) -> Self {
                Self { inner: Some(value) }
            }
        }
    };
}
pub(crate) use option_wrapper;

/// Emits a `#[napi] impl` block exposing a channel receiver as `recv` (async) / `try_recv`,
/// mapping each received `T` into its napi wrapper via `Into`. Two forms:
///
/// - `recv_handler!(Entity => Item)` — an `option_wrapper!` entity whose Zenoh type derefs to
///   `HandlerImpl<T>` (subscribers, queryables, listeners, scout); reached via `get_ref()`.
/// - `recv_handler!(Wrapper.field => Item)` — a newtype holding a `HandlerImpl<T>` in `field`
///   directly (e.g. `ReplyHandler.0`, the receiver returned by one-shot `get`s).
///
/// Lives in a separate impl block because a proc-macro attribute can't expand an inner
/// `macro_rules!` call inside the type's own `#[napi] impl`.
///
/// recv_handler!(Subscriber => Sample);
/// recv_handler!(ReplyHandler.0 => Reply);
macro_rules! recv_handler {
  ($Entity:ident => $Item:ty) => {
    #[napi]
    impl $Entity {
      #[napi]
      pub async fn recv(&self) -> napi::Result<$Item> {
        Ok(self.get_ref()?.recv().await?.into())
      }

      #[napi]
      pub fn try_recv(&self) -> napi::Result<Option<$Item>> {
        Ok(self.get_ref()?.try_recv()?.map(Into::into))
      }
    }
  };
  ($Wrapper:ident . $field:tt => $Item:ty) => {
    #[napi]
    impl $Wrapper {
      #[napi]
      pub async fn recv(&self) -> napi::Result<$Item> {
        Ok(self.$field.recv().await?.into())
      }

      #[napi]
      pub fn try_recv(&self) -> napi::Result<Option<$Item>> {
        Ok(self.$field.try_recv()?.map(Into::into))
      }
    }
  };
}
pub(crate) use recv_handler;

/// Emits an async iterator over a `HandlerImpl<T>`: a `#[napi(async_iterator)]` newtype whose
/// `[Symbol.asyncIterator]` yields each received item's napi wrapper, plus a `stream()` accessor
/// on the owning entity that hands out a shared cursor. Mirrors `recv_handler!`'s two forms:
///
/// - `async_stream!(Entity => Stream yields Wrapper from RawItem)` — an entity whose Zenoh type
///   derefs to `HandlerImpl<RawItem>`; reached via `get_ref()`.
/// - `async_stream!(Wrapper.field => Stream yields Item from RawItem)` — a newtype holding the
///   `HandlerImpl<RawItem>` in `field` directly (e.g. `ReplyHandler.0`).
///
/// `Wrapper` must be a bare identifier, not a path: napi reads the `AsyncGenerator::Yield` type
/// with a strict `Type::Path` match, and a `:ty` metavariable (wrapped in invisible delimiters)
/// parses as `Type::Group`, which silently drops the `[Symbol.asyncIterator]` type-def.
///
/// async_stream!(Subscriber => SampleStream yields Sample from zsample::Sample);
macro_rules! async_stream {
  ($Entity:ident => $Stream:ident yields $Item:ident from $Raw:ty) => {
    #[napi]
    impl $Entity {
      #[napi]
      pub fn stream(&self) -> napi::Result<$Stream> {
        Ok(self.get_ref()?.share().into())
      }
    }
    async_stream!(@def $Stream yields $Item from $Raw);
  };
  ($Wrapper:ident . $field:tt => $Stream:ident yields $Item:ident from $Raw:ty) => {
    #[napi]
    impl $Wrapper {
      #[napi]
      pub fn stream(&self) -> napi::Result<$Stream> {
        Ok(self.$field.share().into())
      }
    }
    async_stream!(@def $Stream yields $Item from $Raw);
  };
  (@def $Stream:ident yields $Item:ident from $Raw:ty) => {
    #[napi(async_iterator)]
    pub struct $Stream(HandlerImpl<$Raw>);

    impl From<HandlerImpl<$Raw>> for $Stream {
      fn from(handler: HandlerImpl<$Raw>) -> Self {
        Self(handler)
      }
    }

    #[napi]
    impl AsyncGenerator for $Stream {
      type Yield = $Item;
      type Next = ();
      type Return = ();

      fn next(
        &mut self,
        _value: Option<()>,
      ) -> impl std::future::Future<Output = napi::Result<Option<$Item>>> + Send + 'static {
        let handler = self.0.share();
        async move { Ok(handler.recv().await.ok().map(Into::into)) }
      }
    }
  };
}
pub(crate) use async_stream;

/// Generates `#[napi(string_enum)] pub enum $Name { variants }` plus both `From` conversions
/// between it and the mirrored `$path` enum. Variant names must match exactly on both sides.
///
/// enum_mapper!(zqos::CongestionControl: Drop, Block, BlockFirst);
macro_rules! enum_mapper {
    ($($seg:ident)::+ : $($variant:ident),+ $(,)?) => {
        $crate::macros::enum_mapper!(@peel $($seg)::+ ; $($seg)::+ ; $($variant),+);
    };
    (@peel $head:ident :: $($tail:ident)::+ ; $full:path ; $($variant:ident),+) => {
        $crate::macros::enum_mapper!(@peel $($tail)::+ ; $full ; $($variant),+);
    };
    (@peel $last:ident ; $full:path ; $($variant:ident),+) => {
        #[napi(string_enum)]
        pub enum $last {
            $($variant,)+
        }

        impl From<$last> for $full {
            fn from(value: $last) -> Self {
                match value {
                    $($last::$variant => Self::$variant,)+
                }
            }
        }

        impl From<$full> for $last {
            fn from(value: $full) -> Self {
                match value {
                    $(<$full>::$variant => Self::$variant,)+
                }
            }
        }

        impl $crate::utils::IntoZenoh for $last {
            type Into = $full;
            fn into_zenoh(self) -> $full {
                self.into()
            }
        }
    };
}
pub(crate) use enum_mapper;

/// Applies optional builder setters in one shot (ported from zenoh-python's `build!`).
/// Each `$value` is an `Option<T: IntoZenoh>` local whose name matches the builder setter;
/// present values are converted via `IntoZenoh` and applied, `None`s are skipped, and the
/// finished builder is returned. Conversions run synchronously as the builder is assembled,
/// so calling `build!(..).await` consumes every input before the first await point.
///
/// build!(session.put(expr, payload), encoding, priority, timestamp);
macro_rules! build {
    ($builder:expr $(, $value:ident)* $(,)?) => {{
        let mut builder = $builder;
        $(
            if let Some(value) = $value.map($crate::utils::IntoZenoh::into_zenoh) {
                builder = builder.$value(value);
            }
        )*
        builder
    }};
}
pub(crate) use build;
