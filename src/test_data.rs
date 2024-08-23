//! Contains a struct that is used for test data.
use std::ops::{Add, AddAssign};

/// A struct that doesn't implement [`Copy`] to use as test data.
#[derive(Debug, Default, Clone)]
pub(crate) struct Data {
    num: i32,
}

impl<T> PartialEq<T> for Data
where
    for<'a> &'a T: Into<i32>,
{
    fn eq(&self, other: &T) -> bool {
        self.num == other.into()
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Data) -> bool {
        self.num == other.num
    }
}

impl AddAssign for Data {
    fn add_assign(&mut self, rhs: Self) {
        self.num += rhs.num;
    }
}

impl<T> Add<T> for Data
where
    T: Into<i32>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        let num: i32 = rhs.into();
        (self.num + num).into()
    }
}

impl From<Data> for i32 {
    fn from(value: Data) -> Self {
        value.num
    }
}

impl From<i32> for Data {
    fn from(value: i32) -> Self {
        Data::new(value)
    }
}

impl Data {
    /// Creates a new [`Data`].
    pub(crate) fn new(num: i32) -> Self {
        Self { num }
    }
}
