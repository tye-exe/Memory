use std::sync::{Arc, MutexGuard};

#[macro_export]
macro_rules! locking_mutate {
    ($($data_access:ident), + $func:expr) => {
        {
            use paste::paste; // Creates new idents
            use $crate::data_access::locking_mutate::data_structures::*; // Newtypes for converting values

            // Acquire mutex lock on both
            // TODO: resolve possible dead-lock
            let ($(paste!{mut [<$data_access _lock>]}, )+) = ($($crate::data_access::locking_mutate::Lock::lock(&$data_access),)+);

            // Executes the given function
            let ($(paste!{[<$data_access _modified>]}, )+) = $func($(
                {
                    // let value = Converter::from(*paste!{[<$data_access _lock>]});
                    // value.custom_into()
                    paste!{[<$data_access _lock>]}.ooa()
                },
            )+);

            // Replaces the internal value with the result
            $(
                *paste!{[<$data_access _lock>]} = {
                    let value = paste!{[<$data_access _modified>]};
                    // value.into()
                    let value = Wrapper::from(value);
                    value.into()
                };
            )+
            // ($({$data_access.replace(Arc::new(paste!{[<$data_access _modified>]}))}, )+);
        }
    };
}

pub trait Lock<Value> {
    type Returns;

    fn lock<'a>(&'a self) -> MutexGuard<'a, Self::Returns>;
}

impl<Value> Lock<Value> for crate::data_access::Oda<Value>
where
    Value: 'static,
{
    type Returns = Option<Arc<Value>>;

    fn lock<'a>(&'a self) -> MutexGuard<'a, Self::Returns> {
        self.current_ref.lock().unwrap()
    }
}

impl<Value> Lock<Value> for crate::data_access::Da<Value>
where
    Value: 'static,
{
    type Returns = Arc<Value>;

    fn lock<'a>(&'a self) -> MutexGuard<'a, Self::Returns> {
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

        locking_mutate!(da_one, da_two |one: Data, two: Data| {
            (one + Data::new(1), two + Data::new(1))
        });

        assert_eq!(da_one.get().as_ref(), &2u64.into());
        assert_eq!(da_two.get().as_ref(), &3u64.into());
    }

    #[test]
    fn oda_lock() {
        let oda_one = Oda::new(Data::new(1));
        let oda_two = Oda::new(Data::new(2));

        locking_mutate!(oda_one, oda_two |one: Option<Data>, two: Option<Data>| {
            (one.map(|value| value + 1), two.map(|value| value + 1))
        });

        assert_eq!(oda_one.get().unwrap().as_ref(), &2u64.into());
        assert_eq!(oda_two.get().unwrap().as_ref(), &3u64.into());
    }
}
