use std::{hash::Hash, ops::RangeBounds};

use ic_stable_structures::{btreemap, BTreeMap, Memory, Storable};

use crate::{BTreeMapIteratorStructure, btreemap::BTreeMapStructure, common::Codec};

/// A versioned BTreeMap.
pub struct VersionedBTreeMap<K, V, C: Codec<V>, M>
where
    K: Storable + Ord + Clone,
    M: Memory,
{
    inner: BTreeMap<K, C, M>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<K, V, C: Codec<V>, M> VersionedBTreeMap<K, V, C, M>
where
    K: Storable + Ord + Clone,
    M: Memory,
{
    /// Create new instance of the VersionedBTreeMap,
    /// overwriting any data structures the memory might have
    /// contained previously
    pub fn new(memory: M) -> Self {
        Self::with_map(BTreeMap::new(memory))
    }

    /// Create new instance of the VersionedBTreeMap.
    ///
    /// PRECONDITION: the memory is either empty or contains a valid
    /// stable BTreeMap.
    pub fn init(memory: M) -> Self {
        Self::with_map(BTreeMap::init(memory))
    }

    /// Create new instance of the VersionedBTreeMap.
    pub fn with_map(map: BTreeMap<K, C, M>) -> Self {
        Self {
            inner: map,
            phantom_v: std::marker::PhantomData,
        }
    }
}

impl<K, V, C: Codec<V>, M> BTreeMapStructure<K, V> for VersionedBTreeMap<K, V, C, M>
where
    K: Storable + Ord + Clone,
    M: Memory,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|v| C::decode(v))
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner
            .insert(key, C::encode(value))
            .map(|v| C::decode(v))
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key).map(|v| C::decode(v))
    }

    fn pop_first(&mut self) -> Option<(K, V)> {
        self.inner
            .pop_first()
            .map(|(k, v)| (k, C::decode(v)))
    }

    fn pop_last(&mut self) -> Option<(K, V)> {
        self.inner
            .pop_last()
            .map(|(k, v)| (k, C::decode(v)))
    }

    fn contains_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    fn first_key_value(&self) -> Option<(K, V)> {
        self.inner
            .first_key_value()
            .map(|(k, v)| (k.clone(), C::decode(v)))
    }

    fn last_key_value(&self) -> Option<(K, V)> {
        self.inner
            .last_key_value()
            .map(|(k, v)| (k.clone(), C::decode(v)))
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

pub struct VersionedBTreeMapIter<'a, K, V, C: Codec<V>, M>(btreemap::Iter<'a, K, C, M>, std::marker::PhantomData<V>)
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory;

impl<K, V, C: Codec<V>, M> Iterator for VersionedBTreeMapIter<'_, K, V, C, M>
where
    K: Storable + Ord + Clone,
    V: Storable,
    M: Memory,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.0.next().map(|entry| {
            let (key, value) = entry.into_pair();
            (key, C::decode(value))
        })
    }
}

impl<K, V, C: Codec<V>, M> BTreeMapIteratorStructure<K, V>
    for VersionedBTreeMap<K, V, C, M>
where
    K: Storable + Clone + Send + Sync + Hash + Eq + PartialEq + Ord,
    V: Storable + Clone + Send + Sync,
    M: Memory,
{
    type Iterator<'a>
        = VersionedBTreeMapIter<'a, K, V, C, M>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        VersionedBTreeMapIter(self.inner.iter(), std::marker::PhantomData)
    }

    fn range(&self, key_range: impl RangeBounds<K>) -> Self::Iterator<'_> {
        VersionedBTreeMapIter(self.inner.range(key_range), std::marker::PhantomData)
    }

    fn iter_from_prev_key(&self, bound: &K) -> Self::Iterator<'_> {
        VersionedBTreeMapIter(self.inner.iter_from_prev_key(bound), std::marker::PhantomData)
    }
}

#[cfg(test)]
mod tests {

    use ic_stable_structures::VectorMemory;

    use crate::{
        test_utils::{Array, UserV1, UserV2, VersionedUser},
    };

    use super::*;

    #[test]
    fn should_use_user_codec() {
        let memory = VectorMemory::default();
        let mut btree_map = BTreeMap::new(memory.clone());

        // The map contains 3 users of different versions
        btree_map.insert(1u32, VersionedUser::V1(UserV1("roger".to_string())));
        btree_map.insert(
            2,
            VersionedUser::V2(UserV2 {
                name: "brian".to_string(),
                age: Some(42),
            }),
        );
        btree_map.insert(3, VersionedUser::V1(UserV1("freddie".to_string())));

        // The map contains 3 users of different versions but VersionedBTreeMap only uses UserV2
        let mut version_map = VersionedBTreeMap::with_map(btree_map);
        version_map.insert(
            1u32,
            UserV2 {
                name: "John".to_string(),
                age: Some(24),
            },
        );

        assert_eq!(
            version_map.get(&1),
            Some(UserV2 {
                name: "John".to_string(),
                age: Some(24)
            })
        );
        assert_eq!(
            version_map.get(&2),
            Some(UserV2 {
                name: "brian".to_string(),
                age: Some(42)
            })
        );
        assert_eq!(
            version_map.get(&3),
            Some(UserV2 {
                name: "freddie".to_string(),
                age: None
            })
        );
    }

    #[test]
    fn should_get_and_insert() {
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default()
        );

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
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default(),
        );
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
        let mut map = VersionedBTreeMap::<u32, u32, u32, _>::new(VectorMemory::default());

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_first(), Some((0, 42)));

        // try to get
        assert!(map.get(&0).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_should_pop_last() {
        let mut map = VersionedBTreeMap::<u32, u32, u32, _>::new(VectorMemory::default());

        map.insert(0u32, 42u32);
        map.insert(10, 100);

        assert_eq!(map.pop_last(), Some((10, 100)));

        assert!(map.get(&10).is_none());

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn should_replace_old_value() {
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _,>::new(
            VectorMemory::default(),
        );

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
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default(),
        );

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
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default(),
        );

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
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default(),
        );

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
        let mut map = VersionedBTreeMap::<u32, u32, u32, _>::new(VectorMemory::default());

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
        let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
            VectorMemory::default(),
        );

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

    #[test]
    fn should_reuse_existing_data_on_init() {
        let memory = VectorMemory::default();
        {
            let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::init(
                memory.clone(),
            );
            map.insert(1, Array([1u8, 1]));
        }

        {
            let map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::init(
                memory,
            );
            assert!(!map.is_empty());
            assert_eq!(Some(Array([1u8, 1])), map.get(&1));
        }
    }

    #[test]
    fn should_erase_existing_data_on_new() {
        let memory = VectorMemory::default();
        {
            let mut map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
                memory.clone(),
            );
            map.insert(1, Array([1u8, 1]));
        }

        {
            let map = VersionedBTreeMap::<u32, Array<2>, Array<2>, _>::new(
                memory,
            );
            assert!(map.is_empty());
            assert_eq!(None, map.get(&1));
        }
    }
}
