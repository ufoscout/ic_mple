use ic_stable_structures::{Memory, BTreeMap, Storable};

use crate::{btreemap::BTreeMapStructure, common::codec::Codec};

/// A versioned BTreeMap.
pub struct VersionedBTreeMap<K, V, D: Clone, M, C: Codec<V, D>> 
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory, {
    inner: BTreeMap<K, V, M>,
    codec: C,
    phantom_d: std::marker::PhantomData<D>,
}

impl<K, V, D: Clone, M, C: Codec<V, D>> VersionedBTreeMap<K, V, D, M, C> 
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory,
    {

    /// Create new instance of the VersionedBTreeMap.
    pub fn new(memory: M, codec: C) -> Self {
        Self::with_map(BTreeMap::new(memory), codec)
    }

    /// Create new instance of the VersionedBTreeMap.
    pub fn with_map(map: BTreeMap<K, V, M>, codec: C) -> Self {
        Self {
            inner: map,
            codec,
            phantom_d: std::marker::PhantomData,
        }
    }

}

impl<K, V, D: Clone, M, C: Codec<V, D>> BTreeMapStructure<K,D> for VersionedBTreeMap<K, V, D, M, C> 
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory, {
        fn get(&self, key: &K) -> Option<D> {
        self.inner.get(key).map(|v| self.codec.decode(v))
    }
    
    fn insert(&mut self, key: K, value: D) -> Option<D> {
        self.inner.insert(key, self.codec.encode(value)).map(|v| self.codec.decode(v))
    }
    
    fn remove(&mut self, key: &K) -> Option<D> {
        self.inner.remove(key).map(|v| self.codec.decode(v))
    }
    
    fn pop_first(&mut self) -> Option<(K, D)> {
        self.inner.pop_first().map(|(k, v)| (k, self.codec.decode(v)))
    }
    
    fn pop_last(&mut self) -> Option<(K, D)> {
        self.inner.pop_last().map(|(k, v)| (k, self.codec.decode(v)))
    }
    
    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }
    
    fn first_key_value(&self) -> Option<(K, D)> {
        self.inner.first_key_value().map(|(k, v)| (k.clone(), self.codec.decode(v)))
    }
    
    fn last_key_value(&self) -> Option<(K, D)> {
        self.inner.last_key_value().map(|(k, v)| (k.clone(), self.codec.decode(v)))
    }
    
    fn len(&self) -> u64 {
        self.inner.len()
    }
    
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    fn clear(&mut self) {
        self.inner.clear_new()
    }
}

#[cfg(test)]
mod tests {

    use ic_stable_structures::VectorMemory;

    use crate::{common::codec::DefaultCodec, test_utils::Array};

    use super::*;

    #[test]
    fn should_get_and_insert() {
        let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

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
        let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());
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
        let mut map =
            VersionedBTreeMap::new(VectorMemory::default(), DefaultCodec::default());

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_first(), Some((0, 42)));

        // try to get
        assert!(map.get(&0).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_should_pop_last() {
        let mut map = CachedStableBTreeMap::new(VectorMemory::default(), 10);

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_last(), Some((10, 100)));

        assert!(map.get(&10).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn should_replace_old_value() {
                let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

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
                let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

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
                let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

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
                let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

        assert_eq!(None, map.insert(1, Array([1u8, 1])));
        assert_eq!(None, map.insert(2, Array([2u8, 1])));
        assert_eq!(None, map.insert(3, Array([3u8, 1])));

        let mut iter = map.iter_upper_bound(&3);
        assert_eq!(iter.next(), Some((2, Array([2u8, 1]))));
        assert_eq!(iter.next(), Some((3, Array([3u8, 1]))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_last_key_value() {
                let mut map =
            VersionedBTreeMap::new(VectorMemory::default(), DefaultCodec::default());

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
                let mut map =
            VersionedBTreeMap::<u32, Array<2>, Array<2>, _, _>::new(VectorMemory::default(), DefaultCodec::default());

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
