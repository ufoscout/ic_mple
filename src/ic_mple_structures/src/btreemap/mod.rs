use std::ops::RangeBounds;

use ic_stable_structures::{btreemap, BTreeMap, Memory, Storable};

pub mod versioned;

pub trait BTreeMapStructure<K, V> {
    /// Return value associated with `key` from stable memory.
    fn get(&self, key: &K) -> Option<V>;

    /// Add or replace value associated with `key` in stable memory.
    ///
    /// # Preconditions:
    ///   - `key.to_bytes().len() <= K::MAX_SIZE`
    ///   - `value.to_bytes().len() <= V::MAX_SIZE`
    fn insert(&mut self, key: K, value: V) -> Option<V>;

    /// Remove value associated with `key` from stable memory.
    ///
    /// # Preconditions:
    ///   - `key.to_bytes().len() <= K::MAX_SIZE`
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Removes and returns the first element in the map.
    fn pop_first(&mut self) -> Option<(K, V)>;

    /// Removes and returns the last element in the map.
    fn pop_last(&mut self) -> Option<(K, V)>;

    /// True if contains the key.
    fn contains_key(&self, key: &K) -> bool;

    /// Returns the first key-value pair in the map.
    fn first_key_value(&self) -> Option<(K, V)>;

    /// Returns the last key-value pair in the map.
    fn last_key_value(&self) -> Option<(K, V)>;

    /// Count of items in the map.
    fn len(&self) -> u64;

    /// Is the map empty.
    fn is_empty(&self) -> bool;

    /// Remove all entries from the map.
    fn clear(&mut self);

}

/// Map that supports ordered iterator
pub trait BTreeMapIteratorStructure<K, V> {
    /// Map iterator type
    type Iterator<'a>: Iterator
    where
        Self: 'a;

    /// Returns iterator over the whole collection
    fn iter(&self) -> Self::Iterator<'_>;

    /// Returns an iterator over the entries in the map where keys
    /// belong to the specified range.
    fn range(&self, key_range: impl RangeBounds<K>) -> Self::Iterator<'_>;

    /// Returns an iterator starting just before the given key.
    ///
    /// Finds the largest key strictly less than `bound` and starts from it.
    /// Useful when `range(bound..)` skips the previous element.
    ///
    /// Returns an empty iterator if no smaller key exists.
    fn iter_from_prev_key(&self, bound: &K) -> Self::Iterator<'_>;
}

impl<K, V, M> BTreeMapStructure<K, V> for BTreeMap<K, V, M>
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    fn pop_first(&mut self) -> Option<(K, V)> {
        self.pop_first()
    }

    fn pop_last(&mut self) -> Option<(K, V)> {
        self.pop_last()
    }

    fn len(&self) -> u64 {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear_new();
    }

    fn contains_key(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    fn first_key_value(&self) -> Option<(K, V)> {
        self.first_key_value()
    }

    fn last_key_value(&self) -> Option<(K, V)> {
        self.last_key_value()
    }
}

impl<K, V, M> BTreeMapIteratorStructure<K, V> for BTreeMap<K, V, M>
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory,
{
    type Iterator<'a>
        = btreemap::Iter<'a, K, V, M>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        self.iter()
    }

    fn range(&self, key_range: impl RangeBounds<K>) -> Self::Iterator<'_> {
        self.range(key_range)
    }

    fn iter_from_prev_key(&self, bound: &K) -> Self::Iterator<'_> {
        self.iter_from_prev_key(bound)
    }
}