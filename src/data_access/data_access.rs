use std::{
    clone::Clone,
    fmt::Debug,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
    thread::sleep,
    time::Duration,
};

// pub fn join()

// locking_mutate!(self.array, self.len, |&mut array, &mut len| {/** code */})

fn a() {
    let oda = Oda::new(1);
    oda.lock().deref();
}

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
    pub(super) current_ref: Arc<Mutex<Option<Arc<Value>>>>,
}

impl<Value> PartialEq for Oda<Value>
where
    Value: PartialEq + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.get(), other.get()) {
            (None, None) => true,
            (None, Some(_)) | (Some(_), None) => false,
            (Some(self_data), Some(other_data)) => *self_data == *other_data,
        }
    }
}

impl<Value> Debug for Oda<Value>
where
    Value: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Oda")
            .field("current_ref", &self.get())
            .finish()
    }
}

impl<Value> From<Value> for Oda<Value>
where
    Value: 'static,
{
    /// Idiomatic to calling [`Self::new(value)`](Self::new()).
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl<Value> Default for Oda<Value>
where
    Value: 'static,
{
    /// Idiomatic to calling [`Self::new(None)`](Self::new()).
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
        Self {
            current_ref: self.current_ref.clone(),
        }
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

    pub fn acquire(value_reference: Arc<Value>) -> Self {
        Self {
            current_ref: Arc::new(Mutex::new(Some(value_reference))),
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

    /// Replaces the the [`Arc`] contained within [`Self`] to the given [`Arc`]. The given [`Arc`] is
    /// held via a strong reference.
    ///
    /// See [`Self::set()`] for more information on the behaviour of current & new references.
    pub fn replace(&self, data_arc: Option<Arc<Value>>) {
        let mut old_data = self.current_ref.lock().unwrap();
        *old_data = data_arc;
    }

    pub fn empty(&self) -> Option<Arc<Value>> {
        let old_arc = self.get();

        let mut old_data = self.current_ref.lock().unwrap();
        *old_data = None;

        old_arc
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
    pub fn mutate<Func>(&self, func: Func)
    where
        Value: Clone,
        Func: FnOnce(Value) -> Value,
    {
        match self.get() {
            Some(old_value) => {
                let mutated_value = func((*old_value).clone());
                self.set(mutated_value);
            }
            None => {}
        }
    }

    fn lock(&self) -> MutexGuard<Option<Arc<Value>>> {
        self.current_ref.lock().unwrap()
    }
}

/// [`DataAccess`](Da)
/// Facilitates "concurrent" reading & writing for a value inside an [`Arc`].
pub struct Da<Value>
where
    Value: 'static,
{
    /// Let me explain; There is an [`Arc`] that points to a value on the heap.
    /// The [`Arc`] can be swapped for a new [`Arc`] pointing to a new value via an immutable reference.
    /// Therefore, the pointer is wrapped in a [`Mutex`], to combat race conditions this would introduce.
    /// The [`Mutex`] is wrapped in an [`Arc`], as clones of [`Da`] must point to the same [`Mutex`].
    pub(super) current_ref: Arc<Mutex<Arc<Value>>>,
}

impl<Value> Debug for Da<Value>
where
    Value: Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Da")
            .field("current_ref", &self.get())
            .finish()
    }
}

impl<Value> Default for Da<Value>
where
    Value: Default + 'static,
{
    /// Idiomatic to calling [`Self::new(value::default())`](Self::new()).
    fn default() -> Self {
        Self {
            current_ref: Arc::new(Mutex::new(Arc::new(Value::default()))),
        }
    }
}

impl<Value> From<Value> for Da<Value>
where
    Value: 'static,
{
    /// Idiomatic to calling [`Self::new(value)`](Self::new()).
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl<Value> Clone for Da<Value>
where
    Value: 'static,
{
    /// Creates a new [`Da`] pointing to the **exact same** value as the original [`Da`].
    fn clone(&self) -> Self {
        Self {
            current_ref: self.current_ref.clone(),
        }
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

    pub fn replace(&self, data_arc: Arc<Value>) {
        let mut old_data = self.current_ref.lock().unwrap();
        *old_data = data_arc;
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
    pub fn mutate<Func>(&self, func: Func)
    where
        Value: Clone,
        Func: FnOnce(Value) -> Value,
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

        use crate::dummy_data::Data;

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

        #[test]
        fn acquire() {
            let original = Oda::new(DummyData::new("A!", 0));
            let acquired = Oda::acquire(original.get().unwrap());

            assert_eq!(original.get(), acquired.get());

            // original has a new internal arc.
            original.set(DummyData::default());

            assert_eq!((*acquired.get().unwrap()), DummyData::new("A!", 0));
            assert_eq!(*original.get().unwrap(), DummyData::default());
        }

        #[test]
        fn replace() {
            let original = Oda::new(DummyData::new("A!", 0));

            let to_replace = Oda::default();
            to_replace.replace(Some(original.get().unwrap()));

            assert_eq!(original.get(), to_replace.get());

            // original has a new internal arc.
            original.set(DummyData::default());

            assert_eq!((*to_replace.get().unwrap()), DummyData::new("A!", 0));
            assert_eq!(*original.get().unwrap(), DummyData::default());
        }

        #[test]
        fn empty() {
            let oda = Oda::new(Data::default());
            assert_eq!(*oda.get().unwrap(), Data::default());

            let empty = oda.empty();
            assert_eq!(*empty.unwrap(), Data::default());

            assert!(oda.get().is_none());
        }
    }
}