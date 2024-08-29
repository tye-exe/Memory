use std::sync::Arc;

use thiserror::Error;

use crate::{
    data_access::{Da, Oda},
    locking_mutate,
};

const EXPECTED_VALUE_MESSAGE: &str = "Expected value inside array bounds";

#[derive(Error, Debug)]
enum CellVecErr {
    #[error("Index out of bounds. Expected {index} (index) < {max_bound}.")]
    OutOfBounds { index: usize, max_bound: usize },
}

struct CellVec<Value>
where
    Value: 'static,
{
    /// The index for the allocated capacity of the array on the heap.
    capacity: Da<usize>,
    /// The current highest index of any inserted value.
    len: Da<usize>,
    /// The array which stores the current values.
    array: Da<Box<[Oda<Value>]>>,
}

impl<Value> Clone for CellVec<Value>
where
    Value: 'static,
{
    fn clone(&self) -> Self {
        CellVec {
            capacity: self.capacity.clone(),
            len: self.len.clone(),
            array: self.array.clone(),
        }
    }
}

impl<Value> IntoIterator for CellVec<Value>
where
    Value: 'static,
{
    type Item = Da<Value>;

    type IntoIter = CellVecIterator<Value>;

    fn into_iter(self) -> Self::IntoIter {
        CellVecIterator {
            cell_vec: self,
            index: 0,
        }
    }
}

impl<Value> CellVec<Value>
where
    Value: 'static,
{
    /// Checks if the given index is within the bounds of the current array length.
    /// Returning `Ok` & `Err` respective of the above statement.
    pub fn in_bounds(&self, index: usize) -> Result<(), CellVecErr> {
        let within_bounds = index < self.len.copy_value();

        match within_bounds {
            true => Ok(()),
            false => Err(CellVecErr::OutOfBounds {
                index,
                max_bound: self.len.copy_value(),
            }),
        }
    }

    /// Creates a new [`CellVec<Value>`] with 0 capacity.
    pub fn new() -> Self {
        Self {
            capacity: Da::new(0),
            len: Da::new(0),
            array: Da::new(Box::new([])),
        }
    }

    /// Returns the value at the given index.
    /// If the given index is outside the bounds of the array None is returned.
    pub fn get(&self, index: usize) -> Option<Arc<Value>> {
        self.in_bounds(index).ok()?;

        Some(
            (**self.array.get())[index]
                .get()
                .expect(EXPECTED_VALUE_MESSAGE),
        )
    }

    /// Sets the given index to the given value, returning the value that was at that index.
    /// If the given index is outside the bounds of the array None is returned.
    pub fn set(&self, index: usize, new_value: Value) -> Option<Arc<Value>> {
        self.in_bounds(index).ok()?;

        let data = (**self.array.get())[index].clone();
        Some(data.set(new_value).expect(EXPECTED_VALUE_MESSAGE))
    }

    pub fn push(&self, new_value: Value) {
        let closure = |mut len: usize, mut capacity: usize, mut array: Box<[Oda<Value>]>| {
            if len >= capacity {
                // Creates clones of every existing value.
                let existing_iter = array.iter().map(|value| (*value).clone());

                // Creates new default Oda's to pad the array.
                let size = capacity.max(1);
                let default_iter = (0..size).map(|_| -> Oda<Value> { Oda::default() });

                array = existing_iter.chain(default_iter).collect();

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

        let (len, capacity, array) = (self.len.clone(), self.capacity.clone(), self.array.clone());

        locking_mutate!(len, capacity, array; closure);
    }

    pub fn remove(&self, index: usize) -> Result<Arc<Value>, CellVecErr> {
        self.in_bounds(index)?;

        let mut removed = None;

        let mut closure = |mut len: usize, mut capacity: usize, mut array: Box<[Oda<Value>]>| {
            let (first, second) = array.split_at(index);
            let (removed_ele, second) = second.split_at(1);

            removed = removed_ele[0].get();

            array = [first, second].concat().into();

            if removed.is_none() {
                return (len, capacity, array);
            }

            len -= 1;

            // if len == 0 {
            //     capacity = 0
            // }

            if capacity >> 1 >= len {
                capacity >>= 1;
            }

            (len, capacity, array)
        };

        let (len, capacity, array) = (self.len.clone(), self.capacity.clone(), self.array.clone());

        locking_mutate!(len, capacity, array; closure);

        Ok(removed.expect(EXPECTED_VALUE_MESSAGE))
    }
}

struct CellVecIterator<Value>
where
    Value: 'static,
{
    cell_vec: CellVec<Value>,
    index: usize,
}

impl<Value> Iterator for CellVecIterator<Value>
where
    Value: 'static,
{
    type Item = Da<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.cell_vec.get(self.index);
        let value = value.map(|value| Da::acquire(value));
        self.index += 1;
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::test_data::Data;

    use super::*;

    fn populate(size: usize) -> CellVec<Data> {
        let cell_vec: CellVec<Data> = CellVec::new();

        for num in 0..size as i32 {
            cell_vec.push(Data::new(num));
        }

        // Sanity check
        assert_eq!(*cell_vec.len.get(), size);
        cell_vec
    }

    #[test]
    fn get_bounds_check() {
        let cell_vec: CellVec<Data> = CellVec::new();
        assert!(cell_vec.get(0).is_none());
        assert!(cell_vec.get(20).is_none());
    }

    #[test]
    fn push_empty() {
        let cell_vec = CellVec::new();
        cell_vec.push(Data::default());
        assert_eq!(*cell_vec.get(0).unwrap(), Data::default());
    }

    #[test]
    fn push() {
        let cell_vec: CellVec<Data> = populate(12);

        for num in 0..12 {
            assert_eq!(*cell_vec.get(num as usize).unwrap(), num.into());
        }
    }

    #[test]
    fn remove_bounds_check() {
        let cell_vec = populate(10);
        // Within array bounds
        assert!(cell_vec.remove(9).is_ok());
        // Out of array bounds
        assert!(cell_vec.remove(9).is_err());
        assert!(cell_vec.remove(21).is_err());
    }

    #[test]
    fn remove_end() {
        let cell_vec = populate(4);
        assert_eq!(*cell_vec.remove(3).unwrap(), 3.into());

        assert_eq!(*cell_vec.get(0).unwrap(), 0.into());
        assert_eq!(*cell_vec.get(1).unwrap(), 1.into());
        assert_eq!(*cell_vec.get(2).unwrap(), 2.into());

        assert!(cell_vec.get(3).is_none());
    }

    #[test]
    fn remove_middle() {
        let cell_vec = populate(4);
        assert_eq!(*cell_vec.remove(1).unwrap(), 1.into());

        assert_eq!(*cell_vec.get(0).unwrap(), 0.into());
        assert_eq!(*cell_vec.get(1).unwrap(), 2.into());
        assert_eq!(*cell_vec.get(2).unwrap(), 3.into());

        assert!(cell_vec.get(3).is_none());
    }

    #[test]
    fn remove_start() {
        let cell_vec = populate(4);
        assert_eq!(*cell_vec.remove(0).unwrap(), 0.into());

        assert_eq!(*cell_vec.get(0).unwrap(), 1.into());
        assert_eq!(*cell_vec.get(1).unwrap(), 2.into());
        assert_eq!(*cell_vec.get(2).unwrap(), 3.into());

        assert!(cell_vec.get(3).is_none());
    }

    #[test]
    fn grows() {
        let cell_vec = CellVec::new();
        assert_eq!(cell_vec.capacity.copy_value(), 0);

        cell_vec.push(Data::default());
        assert_eq!(cell_vec.capacity.copy_value(), 1);

        cell_vec.push(Data::default());
        assert_eq!(cell_vec.capacity.copy_value(), 2);

        cell_vec.push(Data::default());
        assert_eq!(cell_vec.capacity.copy_value(), 4);
        cell_vec.push(Data::default());
        assert_eq!(cell_vec.capacity.copy_value(), 4);

        cell_vec.push(Data::default());
        assert_eq!(cell_vec.capacity.copy_value(), 8);
    }

    #[test]
    fn shirnks() -> Result<(), CellVecErr> {
        let cell_vec = populate(5);
        assert_eq!(cell_vec.capacity.copy_value(), 8usize);

        cell_vec.remove(0)?;
        assert_eq!(cell_vec.capacity.copy_value(), 4usize);
        cell_vec.remove(0)?;
        assert_eq!(cell_vec.capacity.copy_value(), 4usize);

        cell_vec.remove(0)?;
        assert_eq!(cell_vec.capacity.copy_value(), 2usize);

        cell_vec.remove(0)?;
        assert_eq!(cell_vec.capacity.copy_value(), 1usize);

        cell_vec.remove(0)?;
        assert_eq!(cell_vec.capacity.copy_value(), 0usize);

        Ok(())
    }

    #[test]
    fn iterator() {
        let cell_vec = populate(4);
        let mut iter = cell_vec.into_iter();

        assert_eq!(*iter.next().unwrap().get(), 0.into());
        assert_eq!(*iter.next().unwrap().get(), 1.into());
        assert_eq!(*iter.next().unwrap().get(), 2.into());
        assert_eq!(*iter.next().unwrap().get(), 3.into());

        assert!(iter.next().is_none());
    }

    #[test]
    /// The setting function returns the previous value at the index or None
    /// if the index was out of bounds.
    fn setting() {
        let cell_vec = populate(8);

        let set = cell_vec.set(0, 2.into());
        assert_eq!(*set.unwrap(), 0.into());
        assert_eq!(*cell_vec.get(0).unwrap(), 2.into());

        let set = cell_vec.set(5, 2.into());
        assert_eq!(*set.unwrap(), 5.into());
        assert_eq!(*cell_vec.get(5).unwrap(), 2.into());

        // Out of bounds
        assert!(cell_vec.set(20, 2.into()).is_none());
    }

    #[test]
    /// The in_bounds function returns Ok if the index is in bounds & Err if it
    /// is out of bounds.
    fn within_bounds() {
        let cell_vec = populate(5);
        assert!(cell_vec.in_bounds(0).is_ok());
        assert!(cell_vec.in_bounds(4).is_ok());

        assert!(cell_vec.in_bounds(5).is_err_and(|err| {
            format!("{err}") == "Index out of bounds. Expected 5 (index) < 5."
        }));
    }
}
