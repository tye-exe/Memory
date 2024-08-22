use std::ops::{Add, AddAssign};

/// A struct that doesn't implement [`Copy`] to use as test data.
#[derive(Debug, Default, PartialEq, Clone)]
pub(crate) struct Data {
    num: u64,
}

impl AddAssign for Data {
    fn add_assign(&mut self, rhs: Self) {
        self.num += rhs.num;
    }
}

impl Add for Data {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        (self.num + rhs.num).into()
    }
}

impl Add<i32> for Data {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        if rhs.is_negative() {
            return self;
        }

        (self.num + rhs as u64).into()
    }
}

impl<T: Into<u64>> From<T> for Data {
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

impl Data {
    /// Creates a new [`Data`].
    pub(crate) fn new(num: u64) -> Self {
        Self { num }
    }
}
