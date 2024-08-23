//! Contains the [`locking_mutate!()`] macro.
//! See its documentation for more detail.

pub mod data_structures;
// pub mod locking_mutate;

pub use data_structures::*;
// pub use locking_mutate::*;

use std::sync::{Arc, MutexGuard};

#[macro_export]
macro_rules! locking_mutate {
    ($($data_access:ident), +; $func:expr) => {
        {
            // This macro allows for creating new identities within rust code.
            // This is used to create unique local variables during repetitions, otherwise
            // the given identities would have to be shadowed. Making the desired functionality
            // impossible.
            use paste::paste;
            // Contains newtypes for converting values, as this macro has to deal with both
            // the `Data Access (Da)` & `Optional Data Access (Oda)` structs. Which isn't
            // possible as differeing methods have to be used to produce the same outcome
            // for each struct.
            // The newtypes abstract away this behaviour, as the methods used can't be modified
            // at compile time by this macro.
            use $crate::data_access::locking_mutate::data_structures::*;

            // Assigned each acquired mutex lock to unique local variables.
            // TODO: resolve possible dead-lock
            let ($(paste!{mut [<$data_access _lock>]}, )+) = ($($crate::data_access::locking_mutate::Lock::lock(&$data_access),)+);

            // Executes the given function/closure.
            let ($(paste!{[<$data_access _modified>]}, )+) = $func($(
                {
                    // Clones the value out of the `Arc` as the type isn't guaranteed to
                    // implement `Copy`
                    paste!{[<$data_access _lock>]}.ooa()
                },
            )+);

            // Replaces the internal values with returned values from the function/closure.
            $(
                *paste!{[<$data_access _lock>]} = {
                    // Split into separate lines to aid in legibility.
                    let value = paste!{[<$data_access _modified>]};
                    // See above comments for `data_structures` use expression.
                    let value = Wrapper::from(value);
                    value.into()
                };
            )+
        }
    };
}

/// Provides solitary access to data via a [`MutexGuard`].
pub trait Lock<Value> {
    /// The value contained within the returned [`MutexGuard`].
    type Returns;

    /// Returns a [`MutexGuard`] to the underlying data represented by this struct.
    fn lock(&self) -> MutexGuard<'_, Self::Returns>;
}

impl<Value> Lock<Value> for crate::data_access::Oda<Value>
where
    Value: 'static,
{
    type Returns = Option<Arc<Value>>;

    fn lock(&self) -> MutexGuard<'_, Self::Returns> {
        self.current_ref.lock().unwrap()
    }
}

impl<Value> Lock<Value> for crate::data_access::Da<Value>
where
    Value: 'static,
{
    type Returns = Arc<Value>;

    fn lock(&self) -> MutexGuard<'_, Self::Returns> {
        self.current_ref.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_access::{Da, Oda},
        test_data::Data,
    };

    #[test]
    fn da_lock() {
        let da_one = Da::new(Data::new(1));
        let da_two = Da::new(Data::new(2));

        locking_mutate!(da_one, da_two; |one: Data, two: Data| {
            (one + Data::new(1), two + Data::new(1))
        });

        assert_eq!(*da_one.get(), 2.into());
        assert_eq!(*da_two.get(), 3.into());
    }

    #[test]
    fn oda_lock() {
        let oda_one = Oda::new(Data::new(1));
        let oda_two = Oda::new(Data::new(2));

        locking_mutate!(oda_one, oda_two; |one: Option<Data>, two: Option<Data>| {
            (one.map(|value| value + 1), two.map(|value| value + 1))
        });

        assert_eq!(*oda_one.get().unwrap(), 2.into());
        assert_eq!(*oda_two.get().unwrap(), 3.into());
    }
}
