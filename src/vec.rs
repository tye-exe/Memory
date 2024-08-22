use std::sync::Arc;

use crate::data_access::{Da, Oda};

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

impl<Value> CellVec<Value>
where
    Value: 'static,
{
    /// Returns true if the given index is within the allocated bounds of the array.
    fn in_bounds(&self, index: usize) -> bool {
        index < *self.len.get()
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

    // pub fn set(&self, index: usize, new_value: Value) -> Arc<Value> {
    //     let previous_value = self.get(index);
    //     (**self.array.get())[index].set(new_value);
    //     previous_value
    // }

    pub fn push(&self, new_value: Value) {
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

    pub fn remove(&self, index: usize) -> Option<Arc<Value>> {
        let mut removed = None;

        self.array.mutate(|array| {
            removed = array[index].empty();
            array
        });

        // No value was at given index
        if removed.is_none() {
            return removed;
        }

        removed
    }

    // fn test(&self) {
    //     locking_mutate!(self.array, self.len, |&mut array, &mut len| {/** code */})
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn push_empty() {
    //     let cell_vec = CellVec::new();
    //     cell_vec.push(Data::default());
    //     assert_eq!(*cell_vec.get(0), Data::default());
    // }

    // #[test]
    // fn multiple_push() {
    //     let cell_vec: CellVec<Data> = CellVec::new();

    //     for num in 0..12 {
    //         cell_vec.push(num.into());
    //     }

    //     for num in 0..12 {
    //         assert_eq!(*cell_vec.get(num as usize), num.into());
    //     }
    // }

    // // fn set() {}

    // #[test]
    // fn remove() {
    //     let cell_vec = CellVec::new();
    // }
}
