use std::{hash::Hash, ops::RangeBounds};

use ic_stable_structures::{BTreeMap, Memory, Storable};

use crate::{
    BTreeMapIter,
    btreemap::{BTreeMapIteratorStructure, BTreeMapStructure},
    common::LruCache,
};

/// A LRU Cache for BTreeMap
pub struct CachedBTreeMap<K, V, M>
where
    K: Storable + Clone + Send + Sync + 'static + Hash + Eq + PartialEq + Ord,
    V: Storable + Clone + Send + Sync + 'static,
    M: Memory,
{
    inner: BTreeMap<K, V, M>,
    cache: LruCache<K, V>,
}

impl<K, V, M> CachedBTreeMap<K, V, M>
where
    K: Storable + Clone + Send + Sync + 'static + Hash + Eq + PartialEq + Ord,
    V: Storable + Clone + Send + Sync + 'static,
    M: Memory,
{
    /// Create new instance of the CachedUnboundedMap with a fixed number of max cached elements,
    /// overwriting any data structures the memory might have
    /// contained previously.
    pub fn new(memory: M, max_cache_items: u32) -> Self {
        Self::with_map(BTreeMap::new(memory), max_cache_items)
    }

    /// Create new instance of the CachedUnboundedMap with a fixed number of max cached elements.
    ///
    /// PRECONDITION: the memory is either empty or contains a valid
    /// stable BTreeMap.
    pub fn init(memory: M, max_cache_items: u32) -> Self {
        Self::with_map(BTreeMap::init(memory), max_cache_items)
    }

    /// Create new instance of the CachedUnboundedMap with a fixed number of max cached elements.
    pub fn with_map(inner: BTreeMap<K, V, M>, max_cache_items: u32) -> Self {
        Self {
            inner,
            cache: LruCache::new(max_cache_items),
        }
    }

    /// Returns the inner collection so that the caller can have a readonly access to it that bypasses the cache.
    pub fn inner(&self) -> &BTreeMap<K, V, M> {
        &self.inner
    }
}

impl<K, V, M> BTreeMapStructure<K, V> for CachedBTreeMap<K, V, M>
where
    K: Storable + Clone + Send + Sync + 'static + Hash + Eq + PartialEq + Ord,
    V: Storable + Clone + Send + Sync + 'static,
    M: Memory,
{
    fn get(&self, key: &K) -> Option<V> {
        self.cache
            .get_or_insert_with(key, |key| self.inner.get(key))
    }

    /// When a new value is inserted, it is also inserted into the cache; this is
    /// required because caching on the `get` is useless in IC if the method is used in a `query` call
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.cache.insert(key.clone(), value.clone());
        self.inner.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.remove(key);
        self.inner.remove(key)
    }

    fn pop_first(&mut self) -> Option<(K, V)> {
        let (k, v) = self.inner.pop_first()?;
        self.cache.remove(&k);

        Some((k, v))
    }

    fn pop_last(&mut self) -> Option<(K, V)> {
        let (k, v) = self.inner.pop_last()?;
        self.cache.remove(&k);

        Some((k, v))
    }

    fn len(&self) -> u64 {
        self.inner.len()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key) || self.inner.contains_key(key)
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty() && self.inner.is_empty()
    }

    fn clear(&mut self) {
        self.cache.clear();
        self.inner.clear_new()
    }

    /// WARN: this bypasses the cache
    fn first_key_value(&self) -> Option<(K, V)> {
        self.inner.first_key_value()
    }

    /// WARN: this bypasses the cache
    fn last_key_value(&self) -> Option<(K, V)> {
        self.inner.last_key_value()
    }
}

impl<K, V, M> BTreeMapIteratorStructure<K, V> for CachedBTreeMap<K, V, M>
where
    K: Storable + Clone + Send + Sync + Hash + Eq + PartialEq + Ord,
    V: Storable + Clone + Send + Sync,
    M: Memory,
{
    type Iterator<'a>
        = BTreeMapIter<'a, K, V, M>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        BTreeMapIteratorStructure::iter(&self.inner)
    }

    fn range(&self, key_range: impl RangeBounds<K>) -> Self::Iterator<'_> {
        BTreeMapIteratorStructure::range(&self.inner, key_range)
    }

    fn iter_from_prev_key(&self, bound: &K) -> Self::Iterator<'_> {
        BTreeMapIteratorStructure::iter_from_prev_key(&self.inner, bound)
    }
}

#[cfg(test)]
mod tests {
    use ic_stable_structures::VectorMemory;

    use super::*;
    use crate::test_utils::Array;

    #[test]
    fn should_get_and_insert() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert!(map.is_empty());

        assert_eq!(None, map.get(&1));
        assert!(!map.contains_key(&1));
        assert_eq!(None, map.get(&2));
        assert!(!map.contains_key(&2));
        assert_eq!(None, map.get(&3));
        assert!(!map.contains_key(&3));
        assert_eq!(None, map.get(&4));
        assert!(!map.contains_key(&4));

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));
        assert_eq!(3, map.len());

        assert!(!map.is_empty());

        assert_eq!(Some(Array([1u8, 1])), map.get(&1));
        assert!(map.inner.contains_key(&1));
        assert!(map.contains_key(&1));

        assert_eq!(Some(Array([2u8, 1])), map.get(&2));
        assert!(map.contains_key(&2));

        assert_eq!(Some(Array([3u8, 1])), map.get(&3));
        assert!(map.contains_key(&3));

        assert_eq!(None, map.get(&4));
        assert!(!map.contains_key(&4));

        assert_eq!(Some(Array([1u8, 1])), map.insert(1, Array([1u8, 10])));
        assert_eq!(Some(Array([2u8, 1])), map.insert(2, Array([2u8, 10])));
        assert_eq!(3, map.len());

        assert_eq!(Some(Array([2u8, 10])), map.get(&2));

        assert_eq!(Some(Array([1u8, 10])), map.get(&1));

        assert_eq!(Some(Array([3u8, 1])), map.get(&3));

        assert_eq!(None, map.get(&4));

        assert_eq!(Some(Array([1u8, 10])), map.remove(&1));
        assert!(!map.inner.contains_key(&1));
        assert_eq!(None, map.remove(&1));

        assert_eq!(None, map.get(&1));
        assert!(!map.contains_key(&1));

        assert_eq!(Some(Array([2u8, 10])), map.remove(&2));
        assert_eq!(None, map.remove(&2));

        assert_eq!(None, map.get(&2));
        assert!(!map.contains_key(&2));

        assert_eq!(None, map.get(&2));
        assert_eq!(Some(Array([3u8, 1])), map.get(&3));
        assert_eq!(None, map.get(&4));

        assert!(!map.is_empty());

        assert_eq!(Some(Array([3u8, 1])), map.remove(&3));
        assert_eq!(None, map.remove(&3));

        assert_eq!(None, map.get(&3));
        assert!(!map.contains_key(&3));

        assert!(map.is_empty());
    }

    #[test]
    fn should_clear() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));

        assert_eq!(Some(Array([1u8, 1])), map.get(&1));
        assert_eq!(Some(Array([2u8, 1])), map.get(&2));

        map.clear();

        assert_eq!(0, map.len());

        assert_eq!(None, map.get(&1));
        assert_eq!(None, map.get(&2));
    }

    #[test]
    fn test_should_pop_first() {
        let mut map = CachedBTreeMap::new(VectorMemory::default(), 10);

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_first(), Some((0, 42)));

        // try to get
        assert!(map.get(&0).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_should_pop_last() {
        let mut map = CachedBTreeMap::new(VectorMemory::default(), 10);

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_last(), Some((10, 100)));

        assert!(map.get(&10).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn should_replace_old_value() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));
        assert_eq!(3, map.len());

        assert_eq!(Some(Array([1u8, 1])), map.get(&1));
        assert_eq!(Some(Array([2u8, 1])), map.get(&2));

        assert_eq!(Some(Array([1u8, 1])), map.insert(1, Array([1u8, 10])));
        assert_eq!(Some(Array([3u8, 1])), map.insert(3, Array([3u8, 10])));

        assert_eq!(Some(Array([1u8, 10])), map.get(&1));
        assert_eq!(Some(Array([2u8, 1])), map.get(&2));
        assert_eq!(Some(Array([3u8, 10])), map.get(&3));
    }

    #[test]
    fn should_iterate() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));

        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((1, Array([1u8, 1]))));
        assert_eq!(iter.next(), Some((2, Array([2u8, 1]))));
        assert_eq!(iter.next(), Some((3, Array([3u8, 1]))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn should_iterate_over_range() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));

        let mut iter = map.range(2..5);
        assert_eq!(iter.next(), Some((2, Array([2u8, 1]))));
        assert_eq!(iter.next(), Some((3, Array([3u8, 1]))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn should_iterate_upper_bound() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));

        let mut iter = map.iter_from_prev_key(&3);

        assert_eq!(iter.next(), Some((2, Array([2u8, 1]))));
        assert_eq!(iter.next(), Some((3, Array([3u8, 1]))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_last_key_value() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, u32, _>::new(VectorMemory::default(), cache_items);
        assert!(map.is_empty());

        assert!(map.last_key_value().is_none());

        map.insert(0u32, 42u32);
        assert_eq!(map.last_key_value(), Some((0, 42)));

        map.insert(10, 100);
        assert_eq!(map.last_key_value(), Some((10, 100)));

        map.insert(5, 100);
        assert_eq!(map.last_key_value(), Some((10, 100)));

        map.remove(&10);
        assert_eq!(map.last_key_value(), Some((5, 100)));
    }

    #[test]
    fn should_get_and_insert_from_existing_map() {
        let cache_items = 2;
        let mut map = CachedBTreeMap::<u32, Array<2>, _>::new(VectorMemory::default(), cache_items);

        map.inner.insert(1, Array([1u8, 1]));
        map.inner.insert(2, Array([2u8, 1]));

        assert!(map.contains_key(&1));
        assert!(map.contains_key(&2));
        assert!(!map.contains_key(&3));
        assert!(!map.is_empty());

        assert_eq!(Some(Array([1u8, 1])), map.get(&1));

        map.remove(&2);

        assert_eq!(None, map.get(&2));
        assert!(!map.contains_key(&2));
        assert!(!map.inner.contains_key(&2));

        assert!(!map.is_empty());

        map.remove(&1);

        assert_eq!(None, map.get(&1));
        assert!(!map.contains_key(&1));
        assert!(!map.inner.contains_key(&1));

        assert!(map.is_empty());
    }
}
