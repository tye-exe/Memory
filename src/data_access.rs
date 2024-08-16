use std::{
    clone::Clone,
    sync::{Arc, Mutex},
};

/// [`OptionalDataAccess`](Oda):
/// Facilitates "concurrent" reading & writing for an optional value inside an [`Arc`].
pub struct Oda<Value>
where
    Value: 'static,
{
    /// Let me explain; There is an optional [`Arc`] that points to a value on the heap.
    /// The [`Arc`] can be swapped for a new [`Arc`] pointing to a new value via an immutable reference.
    /// Therefore, the pointer is wrapped in a [`Mutex`], to combat race conditions this would introduce.
    /// The [`Mutex`] is wrapped in an [`Arc`], as clones of [`Oda`] must point to the same [`Mutex`].
    current_ref: Arc<Mutex<Option<Arc<Value>>>>,
}

impl<Value> Default for Oda<Value>
where
    Value: 'static,
{
    fn default() -> Self {
        Self {
            current_ref: Arc::new(Mutex::new(None)),
        }
    }
}

impl<Value> Clone for Oda<Value>
where
    Value: 'static,
{
    /// Creates a new [`Oda`] pointing to the **exact same** value as the original [`Oda`].
    fn clone(&self) -> Self {
        let arc = self.current_ref.clone();
        Self { current_ref: arc }
    }
}

impl<Value> Oda<Value>
where
    Value: 'static,
{
    /// Creates a new [`Oda<Value>`].
    pub fn new(data: Value) -> Self {
        Self {
            current_ref: Arc::new(Mutex::new(Some(Arc::new(data)))),
        }
    }

    /// Gets a reference to the current underlying data.
    ///
    /// This reference **will be uneffected** by any subsequent mutations.
    pub fn get(&self) -> Option<Arc<Value>> {
        self.current_ref.lock().unwrap().clone()
    }

    /// Creates new underlying data with the given value; Whilst leaving the old data references uneffected.
    ///
    /// Any subsequent calls to [`get`](Self::get()) will return the new data.
    ///
    /// Any existing references from [`get`](Self::get()) will remain pointing to the old data.
    pub fn set(&self, new_data: Value) {
        let mut old_data = self.current_ref.lock().unwrap();
        *old_data = Some(Arc::new(new_data));
    }

    /// If there is underlying data, it's cloned & the given function will be called with it as the parameter.
    /// The value returned from the function will be set as the new underlying data.
    /// If there is no data then this method **has no effect**.
    ///
    /// This method **does not** hold a lock on the underlying data whilst the given function is executing.
    /// It only waits to acquire a lock when it's reading the value for the initial clone & when writing the
    /// mutated value back to the [`Oda`].
    /// Due to cloning the data out of the [`Oda`], the value passed into the function **is immutable**.
    ///
    /// See [`Self::set()`] for more information on the behaviour of current & new references.
    pub fn mutate<F>(&self, func: F)
    where
        Value: Clone,
        F: FnOnce(Value) -> Value,
    {
        match self.get() {
            Some(old_value) => {
                let mutated_value = func((*old_value).clone());
                self.set(mutated_value);
            }
            None => {}
        }
    }
}

/// [`DataAccess`](Da)
/// Facilitates "concurrent" reading & writing for a value inside an [`Arc`].
#[derive(Default)]
pub struct Da<Value>
where
    Value: 'static,
{
    /// Let me explain; There is an [`Arc`] that points to a value on the heap.
    /// The [`Arc`] can be swapped for a new [`Arc`] pointing to a new value via an immutable reference.
    /// Therefore, the pointer is wrapped in a [`Mutex`], to combat race conditions this would introduce.
    /// The [`Mutex`] is wrapped in an [`Arc`], as clones of [`Da`] must point to the same [`Mutex`].
    current_ref: Arc<Mutex<Arc<Value>>>,
}

impl<Value> Clone for Da<Value>
where
    Value: 'static,
{
    /// Creates a new [`Da`] pointing to the **exact same** value as the original [`Da`].
    fn clone(&self) -> Self {
        let arc = self.current_ref.clone();
        Self { current_ref: arc }
    }
}

impl<Value> Da<Value>
where
    Value: 'static,
{
    /// Creates a new [`Da<Value>`].
    pub fn new(data: Value) -> Self {
        Self {
            current_ref: Arc::new(Mutex::new(Arc::new(data))),
        }
    }

    /// Gets a reference to the current underlying data.
    ///
    /// This reference **will be uneffected** by any subsequent mutations.
    pub fn get(&self) -> Arc<Value> {
        self.current_ref.lock().unwrap().clone()
    }

    /// Creates new underlying data with the given value; Whilst leaving the old data references uneffected.
    ///
    /// Any subsequent calls to [`get`](Self::get()) will return the new data.
    ///
    /// Any existing references from [`get`](Self::get()) will remain pointing to the old data.
    pub fn set(&self, new_data: Value) {
        let mut old_data = self.current_ref.lock().unwrap();
        *old_data = Arc::new(new_data);
    }

    /// Clones the existing underlying data & calls the given function with the clone as the parameter.
    /// The value returned from the function will be set as the new underlying data.
    ///
    /// This method **does not** hold a lock on the underlying data whilst the given function is executing.
    /// It only waits to acquire a lock when it's reading the value for the initial clone & when writing the
    /// mutated value back to the [`Da`].
    /// Due to cloning the data out of the [`Da`], the value passed into the function **is immutable**.
    ///
    /// See [`Self::set()`] for more information on the behaviour of current & new references.
    pub fn mutate<F>(&self, func: F)
    where
        Value: Clone,
        F: FnOnce(Value) -> Value,
    {
        let mutated_value = func((*self.get()).clone());
        self.set(mutated_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct DummyData {
        text: String,
        num: u64,
    }

    impl DummyData {
        fn new(text: &str, num: u64) -> Self {
            let text = text.to_owned();
            Self { text, num }
        }
    }

    #[cfg(test)]
    mod data_access {
        use std::{thread, time::Duration};

        use super::*;

        fn get_default() -> Da<DummyData> {
            Da::new(DummyData::default())
        }

        #[test]
        fn holds_data() {
            let data_access = Da::new(DummyData::default());
            assert_eq!(*data_access.get(), DummyData::default());
        }

        #[test]
        fn mutate_data() {
            let data_access = get_default();
            let mutated_dummy = DummyData::new("I'm different!", 0);

            data_access.set(DummyData::new("I'm different!", 0));
            assert_eq!(*data_access.get(), mutated_dummy);
        }

        #[test]
        fn old_reference() {
            let data_access = get_default();
            let mutated_dummy = DummyData::new("I'm different!", 0);

            // Get a reference to the original data
            let initial_content = data_access.get();

            // Set new data
            data_access.set(DummyData::new("I'm different!", 0));

            assert_eq!(*initial_content, DummyData::default());
            assert_eq!(*data_access.get(), mutated_dummy);
        }

        #[test]
        fn concurrency() {
            let data_access = get_default();
            let data_access_clone = data_access.clone();

            let dummy_data = DummyData::new("I'm different!", 0);
            let dummy_data_clone = DummyData::new("I'm different!", 0);

            // Get a reference to the original data
            let initial_content = data_access.get();

            thread::spawn(move || {
                assert_eq!(*data_access.get(), DummyData::default());

                let mutated_dummy = dummy_data;
                data_access.set(mutated_dummy.clone());

                assert_eq!(*data_access.get(), mutated_dummy);
            })
            .join()
            .unwrap();

            // Original reference still valid.
            assert_eq!(*initial_content, DummyData::default());
            // New references reference new value.
            assert_eq!(*data_access_clone.get(), dummy_data_clone);

            // Arc will get freed after initial_count is dropped.
            assert_eq!(Arc::weak_count(&initial_content), 0);
            assert_eq!(Arc::strong_count(&initial_content), 1);
        }

        #[test]
        fn race_condition() {
            let data_access = get_default();
            let data_access_clone = data_access.clone();

            let dummy_data = DummyData::new("I'm different!", 0);
            let dummy_data_clone = dummy_data.clone();

            // Reference to data before the mutation.
            let before_mutate = data_access_clone.get();
            // Will be re-assigned to the value set during the mutate closure.
            let mut set_value = Arc::new(DummyData::default());

            // Starts mutation
            data_access.mutate(|mut data| {
                data.num += 1;
                assert_eq!(*data_access.get(), DummyData::default());

                // Reads & modifies the value during the mutation.
                {
                    let during_mutate = data_access_clone.get();

                    assert_eq!(*before_mutate, *during_mutate);

                    data_access_clone.set(dummy_data_clone.clone());
                    assert_eq!(*data_access_clone.get(), dummy_data_clone);

                    set_value = data_access_clone.get();
                }

                // The value should be the modified one
                assert_eq!(*data_access.get(), dummy_data);
                // Finishes the mutation.
                data
            });

            // The value now will be the mutation value, as it was the last process to modify the Da.
            assert_eq!(*data_access.get(), DummyData::new("", 1));
            // The pointer to the value set during the mutation will still be valid.
            assert_eq!(*set_value, dummy_data);
        }
    }

    #[cfg(test)]
    mod optional_data_access {
        use std::thread;

        use super::*;

        fn get_default() -> Oda<DummyData> {
            Oda::new(DummyData::default())
        }

        #[test]
        fn default_to_none() {
            let oda: Oda<DummyData> = Oda::default();
            assert!(oda.get().is_none());
        }

        #[test]
        fn holds_data() {
            let data_access = Oda::new(DummyData::default());
            assert_eq!(*data_access.get().unwrap(), DummyData::default());
        }

        #[test]
        fn mutate_data() {
            let data_access = get_default();
            let mutated_dummy = DummyData::new("I'm different!", 0);

            data_access.set(DummyData::new("I'm different!", 0));
            assert_eq!(*data_access.get().unwrap(), mutated_dummy);
        }

        #[test]
        fn old_reference() {
            let data_access = get_default();
            let mutated_dummy = DummyData::new("I'm different!", 0);

            // Get a reference to the original data
            let initial_content = data_access.get().unwrap();

            // Set new data
            data_access.set(DummyData::new("I'm different!", 0));

            assert_eq!(*initial_content, DummyData::default());
            assert_eq!(*data_access.get().unwrap(), mutated_dummy);
        }

        #[test]
        fn concurrency() {
            let data_access = get_default();
            let data_access_clone = data_access.clone();

            let dummy_data = DummyData::new("I'm different!", 0);
            let dummy_data_clone = DummyData::new("I'm different!", 0);

            // Get a reference to the original data
            let initial_content = data_access.get().unwrap();

            thread::spawn(move || {
                assert_eq!(*data_access.get().unwrap(), DummyData::default());

                let mutated_dummy = dummy_data;
                data_access.set(mutated_dummy.clone());

                assert_eq!(*data_access.get().unwrap(), mutated_dummy);
            })
            .join()
            .unwrap();

            // Original reference still valid.
            assert_eq!(*initial_content, DummyData::default());
            // New references reference new value.
            assert_eq!(*data_access_clone.get().unwrap(), dummy_data_clone);

            // Arc will get freed after initial_count is dropped.
            assert_eq!(Arc::weak_count(&initial_content), 0);
            assert_eq!(Arc::strong_count(&initial_content), 1);
        }

        #[test]
        fn race_condition() {
            let data_access = get_default();
            let data_access_clone = data_access.clone();

            let dummy_data = DummyData::new("I'm different!", 0);
            let dummy_data_clone = dummy_data.clone();

            // Reference to data before the mutation.
            let before_mutate = data_access_clone.get().unwrap();
            // Will be re-assigned to the value set during the mutate closure.
            let mut set_value = Arc::new(DummyData::default());

            // Starts mutation
            data_access.mutate(|mut data| {
                data.num += 1;
                assert_eq!(*data_access.get().unwrap(), DummyData::default());

                // Reads & modifies the value during the mutation.
                {
                    let during_mutate = data_access_clone.get().unwrap();

                    assert_eq!(*before_mutate, *during_mutate);

                    data_access_clone.set(dummy_data_clone.clone());
                    assert_eq!(*data_access_clone.get().unwrap(), dummy_data_clone);

                    set_value = data_access_clone.get().unwrap();
                }

                // The value should be the modified one
                assert_eq!(*data_access.get().unwrap(), dummy_data);
                // Finishes the mutation.
                data
            });

            // The value now will be the mutation value, as it was the last process to modify the Da.
            assert_eq!(*data_access.get().unwrap(), DummyData::new("", 1));
            // The pointer to the value set during the mutation will still be valid.
            assert_eq!(*set_value, dummy_data);
        }
    }
}
