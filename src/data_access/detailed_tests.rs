mod test_for_race_condition {
    use std::{
        sync::Arc,
        thread::{self, sleep},
        time::Duration,
    };

    use crate::{
        data_access::{Da, Oda},
        test_data::Data,
    };

    struct CellVec<Value>
    where
        Value: 'static,
    {
        /// The index for the allocated capacity of the array on the heap.
        capacity: Da<usize>,
        /// The current highest index of any inserted value.
        len: Da<usize>,
        array: Da<Box<[Oda<Value>]>>,
    }

    impl<Value> Clone for CellVec<Value>
    where
        Value: Clone + 'static,
    {
        fn clone(&self) -> Self {
            Self {
                capacity: self.capacity.clone(),
                len: self.len.clone(),
                array: self.array.clone(),
            }
        }
    }

    impl<Value> CellVec<Value>
    where
        Value: 'static,
    {
        pub fn new() -> Self {
            Self {
                capacity: Da::new(0),
                len: Da::new(0),
                array: Da::new(Box::new([])),
            }
        }

        pub fn push<Func: FnOnce() -> ()>(&self, new_value: Value, func: Func) {
            // If the array is full then allocate a new one
            if *self.len.get() >= *self.capacity.get() {
                self.array.mutate(|array| {
                    // Creates clones of every existing value.
                    let existing_iter = array.iter().map(|value| (*value).clone());

                    // Creates new default Oda's to pad the array.
                    let size = (*self.capacity.get()).max(1);
                    let default_iter = (0..size).map(|_| -> Oda<Value> { Oda::default() });

                    existing_iter.chain(default_iter).collect()
                });

                func();

                // Double the allocated size
                self.capacity.mutate(|capacity| {
                    if capacity == 0 {
                        capacity + 1
                    } else {
                        capacity * 2
                    }
                });
            }

            // Increment the length
            self.len.mutate(|len| {
                // Set the new value at the current max length.
                (**self.array.get())[len].set(new_value);
                len + 1
            });
        }

        pub fn remove<Func: FnOnce() -> ()>(&self, index: usize, func: Func) -> Option<Arc<Value>> {
            let mut removed = None;

            self.array.mutate(|mut array| {
                // removed = array[index].empty();

                let (first, second) = array.split_at(index);
                let (removed_ele, second) = second.split_at(1);

                removed = removed_ele[0].get();

                array = [first, second].concat().into();

                // for element_index in (index + 1..*self.capacity.get()) {
                //     array[element_index - 1] = array[element_index];
                // }

                // let mut iter = end.into_iter();
                // let _ = iter.next();
                // array = start.into_iter().chain(iter).collect();

                array
            });

            func();

            // No value was at given index
            if removed.is_none() {
                return removed;
            }

            self.len.mutate(|len| len - 1);

            // Half the allocated size
            if *self.capacity.get() - *self.len.get() < *self.len.get() {
                self.capacity.mutate(|capacity| capacity >> 1);
            }

            removed
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is 3")]
    fn encounter_race_condition() {
        let cell_vec = CellVec::new();
        cell_vec.push(Data::default(), || {});

        // Check that the Vec expand
        assert_eq!(cell_vec.capacity.get().as_ref(), &1usize);
        assert_eq!(cell_vec.len.get().as_ref(), &1usize);

        cell_vec.push(Data::default(), || {});
        cell_vec.push(Data::default(), || {});

        assert_eq!(cell_vec.capacity.get().as_ref(), &4usize);
        assert_eq!(cell_vec.len.get().as_ref(), &3usize);

        let (remove_tx, remove_rx) = oneshot::channel();
        let clone = cell_vec.clone();

        let remove = thread::spawn(move || {
            clone.remove(2, || {
                remove_tx.send(()).unwrap();
                sleep(Duration::from_millis(100));
            });
        });

        // Ensures code runs in a consistent order
        remove_rx.recv().unwrap();

        cell_vec.push(Data::default(), || {
            sleep(Duration::from_millis(300));
        });

        remove.join().unwrap();
    }
}

mod no_race_condition {
    use std::{
        sync::Arc,
        thread::{self, sleep},
        time::Duration,
    };

    use crate::{
        data_access::{Da, Oda},
        locking_mutate,
        test_data::Data,
    };

    struct CellVec<Value>
    where
        Value: 'static,
    {
        /// The index for the allocated capacity of the array on the heap.
        capacity: Da<usize>,
        /// The current highest index of any inserted value.
        len: Da<usize>,
        array: Da<Box<[Oda<Value>]>>,
    }

    impl<Value> Clone for CellVec<Value>
    where
        Value: Clone + 'static,
    {
        fn clone(&self) -> Self {
            Self {
                capacity: self.capacity.clone(),
                len: self.len.clone(),
                array: self.array.clone(),
            }
        }
    }
    impl<Value> CellVec<Value>
    where
        Value: 'static,
    {
        pub fn new() -> Self {
            Self {
                capacity: Da::new(0),
                len: Da::new(0),
                array: Da::new(Box::new([])),
            }
        }

        pub fn push<Func: FnOnce() -> ()>(&self, new_value: Value, func: Func) {
            let closure = |mut len: usize, mut capacity: usize, mut array: Box<[Oda<Value>]>| {
                if len >= capacity {
                    // Creates clones of every existing value.
                    let existing_iter = array.iter().map(|value| (*value).clone());

                    // Creates new default Oda's to pad the array.
                    let size = capacity.max(1);
                    let default_iter = (0..size).map(|_| -> Oda<Value> { Oda::default() });

                    array = existing_iter.chain(default_iter).collect();

                    func();

                    if capacity == 0 {
                        capacity += 1;
                    } else {
                        capacity <<= 1;
                    }
                }

                array[len].set(new_value);

                len += 1;
                (len, capacity, array)
            };

            let (len, capacity, array) =
                (self.len.clone(), self.capacity.clone(), self.array.clone());

            locking_mutate!(
                len,
                capacity,
                array; closure);
        }

        pub fn remove<Func: FnOnce() -> ()>(&self, index: usize, func: Func) -> Option<Arc<Value>> {
            let mut removed = None;

            let closure = |mut len: usize, mut capacity: usize, mut array: Box<[Oda<Value>]>| {
                let (first, second) = array.split_at(index);
                let (removed_ele, second) = second.split_at(1);

                removed = removed_ele[0].get();

                array = [first, second].concat().into();

                func();

                if removed.is_none() {
                    return (len, capacity, array);
                }

                len -= 1;

                if capacity - len < len {
                    capacity >>= 1;
                }

                (len, capacity, array)
            };

            let (len, capacity, array) =
                (self.len.clone(), self.capacity.clone(), self.array.clone());

            locking_mutate!(
                len,
                capacity,
                array; closure);

            removed
        }
    }

    #[test]
    fn avoid_race_condition() {
        let cell_vec = CellVec::new();
        cell_vec.push(Data::default(), || {});

        // Check that the Vec expand
        assert_eq!(cell_vec.capacity.get().as_ref(), &1usize);
        assert_eq!(cell_vec.len.get().as_ref(), &1usize);

        cell_vec.push(Data::default(), || {});
        cell_vec.push(Data::default(), || {});

        assert_eq!(cell_vec.capacity.get().as_ref(), &4usize);
        assert_eq!(cell_vec.len.get().as_ref(), &3usize);

        let (remove_tx, remove_rx) = oneshot::channel();
        let clone = cell_vec.clone();

        let remove = thread::spawn(move || {
            clone.remove(2, || {
                remove_tx.send(()).unwrap();
                sleep(Duration::from_millis(100));
            });
        });

        // Ensures threads run in consistent order
        remove_rx.recv().unwrap();

        let clone = cell_vec.clone();
        let push = thread::spawn(move || {
            clone.push(Data::default(), || {
                sleep(Duration::from_millis(300));
            });
        });

        remove.join().unwrap();
        push.join().unwrap();

        // The remove & push "cancel out"
        assert_eq!(cell_vec.capacity.get().as_ref(), &4usize);
        assert_eq!(cell_vec.len.get().as_ref(), &3usize);
    }
}
