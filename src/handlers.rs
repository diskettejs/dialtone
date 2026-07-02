use std::marker::PhantomData;

pub(crate) enum HandlerImpl<T> {
  Rust(PhantomData<T>),
  // JavaScript(Object<'static>),
}
