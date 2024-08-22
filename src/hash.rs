use std::{fmt::Debug, hash::Hash, sync::Arc};

use crate::data_access::{Da, Oda};

const DEFAULT_MAX_SIZE: u64 = 256;

pub struct CellHashMap<Key, Value>
where
    Key: Debug + Hash + Clone + Eq + 'static,
    Value: Debug + Clone + 'static,
{
    // current_size: Da<usize>,
    array: Da<[Oda<CellEntry<Key, Value>>; DEFAULT_MAX_SIZE as usize]>,
    // data: Da<Contains<Key, Value>>,
}

impl<Key, Value> CellHashMap<Key, Value>
where
    Key: Debug + Hash + Clone + Eq + 'static,
    Value: Debug + Clone + 'static,
{
    pub fn new() -> Self {
        Self {
            // data: Contains {
            //     element_count: 0,
            //     array: core::array::from_fn(|_| Oda::default()),
            // }
            // .into(),
            // current_size: 0.into(),
            array: core::array::from_fn(|_| Oda::default()).into(),
        }
    }

    // fn array_get(&self, index: usize) -> Option<Arc<CellEntry<Key, Value>>> {
    //     self.data.get().array[index].get()
    // }

    // fn size_get(&self) -> usize {
    //     self.data.get().element_count
    // }

    pub fn put(&self, key: Key, value: Value) -> Option<Value> {
        let hash_val: u64 = hash_key(key.clone());
        let position = (hash_val % DEFAULT_MAX_SIZE) as usize;

        let mut result = None;

        match self.array.get().as_ref()[position].get() {
            Some(entry) => result = entry.set(CellEntry::new(key, value)),
            None => self.array.get().as_ref()[position].set(CellEntry::new(key, value)),
        }

        // If the result was none then a new value was added.
        if result.is_none() {
            // self.current_size.mutate(|size| size + 1)
        }
        result.map(|value| value.value.get().as_ref().clone())
    }

    pub fn get(&self, key: Key) -> Option<Arc<Value>> {
        let hash_val: u64 = hash_key(key.clone());
        let position = (hash_val % DEFAULT_MAX_SIZE) as usize;

        match self.array.get().as_ref()[position].get() {
            Some(data) => data.get(&key).map(|cell_entry| cell_entry.value.get()),
            None => None,
        }
    }

    pub fn remove(&self, key: Key) {
        let hash_val: u64 = hash_key(key.clone());
        let position = (hash_val % DEFAULT_MAX_SIZE) as usize;

        let entry_iter = self.array.get().as_ref()[position].get().map(|root| {
            // It makes the compiler happy.
            (*root)
                .clone()
                // Removes the element if the key matches.
                .filter(|entry| {
                    println!("{:?}", entry.key);
                    println!("{:?}", key);
                    entry.key != key
                })
        });

        println!("{:#?}", entry_iter);

        if let Some(mut iter) = entry_iter {
            if let Some(mut root) = iter.next() {
                let mut collection = Vec::new();
                for entry in iter {
                    // println!("\n\nentry: {:?}\n\n", entry);
                    collection.push(entry);
                }
                collection.reverse();

                // Line causing issue?
                // let mut collect: Vec<Arc<CellEntry<Key, Value>>> = iter.collect();
                // collect.reverse();

                // Re-creates the linked list.
                for entry in collection {
                    entry.next.replace(root);
                    root = entry;
                }

                // Updates the array
                self.array.mutate(|array| {
                    array[position].replace(root);
                    array
                });
            };
        };
    }
}

#[derive(Clone, Debug)]
pub struct CellEntry<Key, Value>
where
    Key: Debug + Eq + Clone + 'static,
    Value: Debug + Clone + 'static,
{
    key: Key,
    value: Da<Value>,
    next: Oda<Self>,
}

// impl<Key, Value> FromIterator<Arc<Self>> for CellEntry<Key, Value>
// where
//     Key: Eq + Clone + 'static,
//     Value: Clone + 'static,
// {
//     fn from_iter<T: IntoIterator<Item = Arc<Self>>>(iter: T) -> Self {
//         let iter = iter.into_iter();
//         match iter.next() {
//             Some(base) => {}
//             None => {}
//         }
//     }
// }

// impl<Key, Value> Iterator for CellEntry<Key, Value>
// where
//     Key: Debug + Eq + Clone + 'static,
//     Value: Debug + Clone + 'static,
// {
//     type Item = Arc<Self>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.next.get().clone()
//     }
// }

impl<Key, Value> CellEntry<Key, Value>
where
    Key: Debug + Eq + Clone + 'static,
    Value: Debug + Clone + 'static,
{
    /// Creates a new [`CellKeyValue<Key, Value>`].
    pub fn new(key: Key, value: Value) -> Self {
        Self {
            key,
            value: Da::new(value),
            next: Oda::default(),
        }
    }

    pub fn set(&self, entry: Self) -> Option<Self> {
        match (self.key == entry.key, self.next.get()) {
            // This matches.
            (true, _next) => {
                let cell_entry = Some(Self::new(entry.key, (*self.value.get()).clone()));
                self.value.set((*entry.value.get()).clone());
                cell_entry
            }
            // This doesn't match but the next might.
            (false, Some(_)) => {
                self.next.get().and_then(|value| value.set(entry))
                // next.set(entry)
            }
            // None of the keys matched; The entry is new.
            (false, None) => {
                self.next.set(entry);
                None
            }
        }
    }

    // pub fn remove(&self, key: Key) -> Option<Self> {
    // match (self.key == key, self.next.get()) {
    //     // This matches.
    //     (true, next) => {
    //         let cell_entry = Some(Self::new(entry.key, (*self.value.get()).clone()));
    //         self.value.set((*entry.value.get()).clone());
    //         cell_entry;
    //     }
    //     // This doesn't match but the next might.
    //     (false, Some(_)) => {
    //         self.next.get().and_then(|value| value.set(entry))
    //         // next.set(entry)
    //     }
    //     // None of the keys matched; The entry is new.
    //     (false, None) => {
    //         self.next.set(entry);
    //         None
    //     }
    // }
    // }

    pub fn get(&self, key: &Key) -> Option<Self> {
        match (self.key == *key, self.next.get()) {
            (true, _) => Some(self.clone()),
            (false, None) => None,
            (false, Some(next)) => (*next).get(key),
        }
    }
}

fn hash_key<Key: Hash>(key: Key) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    key.hash(&mut hasher);
    let hash_val = std::hash::Hasher::finish(&hasher);
    hash_val
}

#[cfg(test)]
mod tests {
    // use core::panic;
    // use rand::distributions::{Alphanumeric, DistString};

    use core::panic;
    use std::{
        thread::{self, sleep},
        time::Duration,
    };

    use super::*;
    use crate::dummy_data::*;

    #[test]
    fn retrieve_value() {
        let cell_hash_map = CellHashMap::new();

        cell_hash_map.put("test", Data::default());
        let data = cell_hash_map.get("test").unwrap();

        assert_eq!(*data, Data::default());
    }

    #[test]
    fn overwrite() {
        let cell_hash_map = CellHashMap::new();

        cell_hash_map.put("test", Data::default());
        let data_one = cell_hash_map.get("test").unwrap();

        let put = cell_hash_map.put("test", 1u64.into());
        assert_eq!(*data_one, put.unwrap());
        let data_two = cell_hash_map.get("test").unwrap();

        assert_eq!(*data_two, 1u64.into());
    }

    #[test]
    fn lifetime() {
        let cell_hash_map = CellHashMap::new();

        cell_hash_map.put("test", Data::default());
        let data_one = cell_hash_map.get("test").unwrap();

        {
            let replaced = cell_hash_map.put("test", 1u64.into());
            assert_eq!(*data_one, replaced.unwrap());
            // replaced dropped
        }
        assert_eq!(*data_one, Data::default());

        let data_two = cell_hash_map.get("test").unwrap();

        assert_eq!(*data_two, 1u64.into());
    }

    fn hash_scoped(string: &str) -> u64 {
        hash_key(string) % DEFAULT_MAX_SIZE
    }

    // Compute same hashes:
    // let hash = hash_key("test") % DEFAULT_MAX_SIZE;
    // loop {
    //     let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    //     let rand_hash = hash_key(string.as_str()) % DEFAULT_MAX_SIZE;
    //     if rand_hash == hash {
    //         println!("{}", string);
    //         panic!("A")
    //     }
    // }

    #[test]
    fn remove() {
        let test = "test";
        // Hashes match
        assert_eq!(hash_scoped(test), hash_scoped("CQPqhZW1srzeR3hU"));
        assert_eq!(hash_scoped(test), hash_scoped("6KegZ36lLDl73Ke9"));
        assert_eq!(hash_scoped(test), hash_scoped("JDbtrFT83atStP2B"));
        assert_eq!(hash_scoped(test), hash_scoped("QWT6GYpvFZxpqTzd"));
        // Hashes don't match
        assert_ne!(hash_scoped(test), hash_scoped("a"));
        assert_ne!(hash_scoped(test), hash_scoped("b"));
        assert_ne!(hash_scoped(test), hash_scoped("d"));
        assert_ne!(hash_scoped(test), hash_scoped("e"));

        let cell_hash_map = CellHashMap::new();

        cell_hash_map.put(test, Data::default());
        cell_hash_map.put("CQPqhZW1srzeR3hU", Data::new(1));
        cell_hash_map.put("JDbtrFT83atStP2B", Data::new(2));
        cell_hash_map.put("6KegZ36lLDl73Ke9", Data::new(3));
        cell_hash_map.put("QWT6GYpvFZxpqTzd", Data::new(4));
        cell_hash_map.put("a", Data::new(5));
        cell_hash_map.put("b", Data::new(6));
        cell_hash_map.put("d", Data::new(7));
        cell_hash_map.put("e", Data::new(8));

        // Other key uneffected
        // cell_hash_map.remove("e");
        // assert_eq!(*cell_hash_map.get("test").unwrap(), Data::default());

        // Double remove has no effect
        // cell_hash_map.remove("e");
        // assert_eq!(*cell_hash_map.get("test").unwrap(), Data::default());

        cell_hash_map.remove("6KegZ36lLDl73Ke9");
        assert_eq!(*cell_hash_map.get("test").unwrap(), Data::default());
    }
}
