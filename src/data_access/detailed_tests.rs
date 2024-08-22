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

    pub fn get(&self, index: usize) -> Option<Arc<Value>> {
        (**self.array.get())[index].get()
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

        self.array.mutate(|array| {
            removed = array[index].empty();
            array
        });

        func();

        // No value was at given index
        if removed.is_none() {
            return removed;
        }

        // Half the allocated size
        self.capacity.mutate(|capacity| capacity >> 1);

        removed
    }
}

// #[test]
// fn race_condition() {
//     let cell_vec = CellVec::new();
//     cell_vec.push(Data::default(), || {});

//     // Check that the Vec expand
//     assert_eq!(cell_vec.capacity.get().as_ref(), &1usize);
//     assert_eq!(cell_vec.len.get().as_ref(), &1usize);

//     cell_vec.push(Data::default(), || {});
//     cell_vec.push(Data::default(), || {});

//     assert_eq!(cell_vec.capacity.get().as_ref(), &4usize);
//     assert_eq!(cell_vec.len.get().as_ref(), &3usize);

//     let clone = cell_vec.clone();
//     let remove = thread::spawn(move || {
//         clone.remove(2, || {
//             sleep(Duration::from_millis(100));
//         });
//     });

//     let clone = cell_vec.clone();
//     let push = thread::spawn(move || {
//         clone.push(Data::default(), || {
//             sleep(Duration::from_millis(300));
//         });
//     });

//     remove.join().unwrap();
//     push.join().unwrap();

//     // The remove & push "cancel out".
//     assert_eq!(cell_vec.capacity.get().as_ref(), &4usize);
//     assert_eq!(cell_vec.len.get().as_ref(), &3usize);
// }
