//! Contains the Newtypes to allow for type conversion within [`locking_mutate`](super::super::locking_mutate::locking_mutate)
use std::sync::Arc;

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

pub trait OutOfArc {
    type Output;

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
