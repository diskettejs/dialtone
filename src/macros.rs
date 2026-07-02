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

        impl $crate::handlers::IntoZenoh for $last {
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
            if let Some(value) = $value.map($crate::handlers::IntoZenoh::into_zenoh) {
                builder = builder.$value(value);
            }
        )*
        builder
    }};
}
pub(crate) use build;
