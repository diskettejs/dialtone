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

        impl $crate::options::IntoZenoh for $last {
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
            if let Some(value) = $value.map($crate::options::IntoZenoh::into_zenoh) {
                builder = builder.$value(value);
            }
        )*
        builder
    }};
}
pub(crate) use build;

/// Generates a standalone `#[napi] impl $Struct { ... }` with the 11
/// `crate::handlers::FifoChannelHandler<T>`-forwarding methods. Coexists with a
/// hand-written impl block for the same struct via napi-rs's additive, TypeId-keyed
/// class registry.
///
/// channel_forward!(Subscriber, receiver, Sample);
/// channel_forward!(Replies, inner, Reply);
macro_rules! channel_forward {
    ($Struct:ident, $field:ident, $J:ty) => {
        #[napi]
        impl $Struct {
            #[napi]
            pub async fn recv(&self) -> napi::Result<$J> {
                self.$field.recv::<$J>().await
            }

            #[napi]
            pub fn try_recv(&self) -> napi::Result<Option<$J>> {
                self.$field.try_recv::<$J>()
            }

            #[napi]
            pub fn drain(&self) -> Vec<$J> {
                self.$field.drain::<$J>()
            }

            #[napi]
            pub fn is_disconnected(&self) -> bool {
                self.$field.is_disconnected()
            }

            #[napi]
            pub fn is_empty(&self) -> bool {
                self.$field.is_empty()
            }

            #[napi]
            pub fn is_full(&self) -> bool {
                self.$field.is_full()
            }

            #[napi]
            pub fn len(&self) -> u32 {
                self.$field.len()
            }

            #[napi]
            pub fn capacity(&self) -> Option<u32> {
                self.$field.capacity()
            }

            #[napi]
            pub fn sender_count(&self) -> u32 {
                self.$field.sender_count()
            }

            #[napi]
            pub fn receiver_count(&self) -> u32 {
                self.$field.receiver_count()
            }

            #[napi]
            pub fn stream<'env>(&self, env: &'env Env) -> napi::Result<ReadableStream<'env, $J>> {
                self.$field.stream::<$J>(env)
            }
        }
    };
}
pub(crate) use channel_forward;
