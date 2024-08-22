//! Contains newtypes for converting types within the [`locking_mutate`](lm) macro.
//!
//! Newtypes must be used as abstract away the type conversion, as the [`locking_mutate`](lm)
//! macro operates on [`Oda`] & [`Da`] values, which require different methods to provide the
//! same functionality.
//! And declerative macros don't have the capability to compile different based upon the given
//! type.

// Used for module doc links.
#[allow(unused_imports)]
use crate::data_access::{locking_mutate::locking_mutate as lm, Da, Oda};

use std::sync::Arc;

/// Used to convert between types, see module comments.
pub struct Converter<Value>(Option<Arc<Value>>);

impl<Value> From<Option<Value>> for Converter<Value> {
    fn from(value: Option<Value>) -> Self {
        Converter(value.map(|value| Arc::new(value)))
    }
}

impl<Value> From<Value> for Converter<Value> {
    fn from(value: Value) -> Self {
        Converter(Some(Arc::new(value)))
    }
}

impl<Value: Clone> Into<Option<Value>> for Converter<Value> {
    fn into(self) -> Option<Value> {
        self.0.map(|value| (*value).clone())
    }
}

/// Returns a value from inside an [`Arc`].
pub trait OutOfArc {
    /// The output value.
    type Output;

    /// Returns [`Self::Output`](OutOfArc::Output) from inside an [`Arc`].
    fn ooa(&self) -> Self::Output;
}

impl<T: Clone> OutOfArc for Option<Arc<T>> {
    type Output = Option<T>;

    fn ooa(&self) -> Self::Output {
        self.as_ref().map(|val| (**val).clone())
    }
}

impl<T: Clone> OutOfArc for Arc<T> {
    type Output = T;

    fn ooa(&self) -> Self::Output {
        (**self).clone()
    }
}

/// Used to convert between types, see module comments.
pub struct Wrapper<Data> {
    data: Data,
}

impl<T> From<T> for Wrapper<T> {
    fn from(data: T) -> Self {
        Wrapper { data }
    }
}

impl<T> From<Wrapper<Option<T>>> for Option<Arc<T>> {
    fn from(value: Wrapper<Option<T>>) -> Self {
        value.data.map(|value| Arc::new(value))
    }
}
impl<T> From<Wrapper<Option<T>>> for Arc<T> {
    fn from(value: Wrapper<Option<T>>) -> Self {
        Arc::new(value.data.unwrap())
    }
}

impl<T> From<Wrapper<T>> for Arc<T> {
    fn from(value: Wrapper<T>) -> Self {
        Arc::new(value.data)
    }
}
