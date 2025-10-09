use std::cmp::min;
use std::mem::size_of;
use std::num::NonZeroU64;

use ic_stable_structures::storable::Bound;
use ic_stable_structures::{Memory, StableCell, Storable};

use crate::vec::VecStructure;
use crate::vec::ext::VecExt;

/// Ring buffer indices state
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StableRingBufferIndices {
    /// Index of the first element in the buffer
    start: u64,
    /// Number of elements in the buffer
    len: u64,
    /// Capacity of the buffer
    capacity: u64,
}

impl StableRingBufferIndices {
    /// Create a new Indices with the provided capacity
    pub fn new(capacity: NonZeroU64) -> Self {
        Self {
            start: 0,
            len: 0,
            capacity: capacity.get(),
        }
    }

    /// Converts an offset from the start element into an index.
    ///
    /// There is no garantee that an element present at the returned index.
    fn offset_to_index(&self, offset: u64) -> u64 {
        (self.start + offset) % self.capacity
    }

    /// Index of the element placed with the `n` offset from start.
    fn nth_element(&self, n: u64) -> Option<u64> {
        (n < self.len).then_some((self.start + n) % self.capacity)
    }

    /// Index of the element placed with the `n` offset from end.
    fn nth_element_from_end(&self, index: u64) -> Option<u64> {
        let index_from_start = self.len.checked_sub(index + 1)?;
        self.nth_element(index_from_start)
    }

    /// Returns the number of elements in the buffer
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Increases number of elements.
    /// The `capacity` is the upper bound.
    pub fn increase_len(&mut self, by: u64) {
        self.len = min(self.len + by, self.capacity);
    }

    /// Decreases number of elements.
    /// `0`` is the lower bound.
    pub fn decrease_len(&mut self, arg: u64) {
        self.len = self.len.saturating_sub(arg);
    }

    /// Increases first element index.
    /// Will wrap if increased start value >= `capacity`.
    pub fn increase_start(&mut self, by: u64) {
        self.start = self.offset_to_index(by);
    }

    /// Returns the capacity of the buffer
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns whether is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

const STABLE_RING_BUFFER_INDICES_SIZE: usize = 3 * size_of::<u64>();

impl Storable for StableRingBufferIndices {
    const BOUND: Bound = Bound::Bounded {
        max_size: STABLE_RING_BUFFER_INDICES_SIZE as u32,
        is_fixed_size: true,
    };

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        let mut buf = Vec::with_capacity(STABLE_RING_BUFFER_INDICES_SIZE);
        buf.extend_from_slice(&self.start.to_le_bytes());
        buf.extend_from_slice(&self.len.to_le_bytes());
        buf.extend_from_slice(&self.capacity.to_le_bytes());
        buf.into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self {
            start: u64::from_le_bytes(bytes[..8].try_into().expect("first: expected 8 bytes")),
            len: u64::from_le_bytes(bytes[8..16].try_into().expect("latest: expected 8 bytes")),
            capacity: u64::from_le_bytes(
                bytes[16..24]
                    .try_into()
                    .expect("capacity: expected 8 bytes"),
            ),
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into()
    }
}

/// Stable ring buffer implementation
pub struct StableRingBuffer<T: Storable + Clone, DataMemory: Memory, IndicesMemory: Memory> {
    /// Vector with elements
    data: VecExt<T, DataMemory>,
    /// Indices that specify where are the first and last elements in the buffer
    indices: StableCell<StableRingBufferIndices, IndicesMemory>,
}

impl<T: Storable + Clone, DataMemory: Memory, IndicesMemory: Memory>
    StableRingBuffer<T, DataMemory, IndicesMemory>
{
    /// Creates new ring buffer
    pub fn new(
        data_memory: DataMemory,
        indices_memory: IndicesMemory,
        default_history_size: NonZeroU64,
    ) -> Self {
        Self {
            data: VecExt::new(data_memory),
            indices: StableCell::new(
                indices_memory,
                StableRingBufferIndices::new(default_history_size),
            ),
        }
    }

    /// Creates new ring buffer
    pub fn new_with(
        data: VecExt<T, DataMemory>,
        indices: StableCell<StableRingBufferIndices, IndicesMemory>,
    ) -> Self {
        Self { data, indices }
    }

    /// Removes all elements in the buffer
    pub fn clear(&mut self) {
        self.with_indices_data_mut(|indices, data| {
            *indices = StableRingBufferIndices::new(
                indices
                    .capacity()
                    .try_into()
                    .expect("capacity should be non-zero"),
            );
            data.clear();
        });
    }

    /// Number of elements in the buffer
    pub fn len(&self) -> u64 {
        self.indices.get().len
    }

    /// Returns whether is empty
    pub fn is_empty(&self) -> bool {
        self.indices.get().len == 0
    }

    /// Max capacity of the buffer
    pub fn capacity(&self) -> u64 {
        self.indices.get().capacity
    }

    /// Update the ring buffer capacity to the given value.
    /// The elements that do not fit into new capacity will be deleted.
    ///
    /// This operataion performs a copy of all elements that need to be preserved.
    /// This may be inefficient if there are a lot of elements.
    pub fn resize(&mut self, new_capacity: NonZeroU64) {
        self.with_indices_data_mut(|indices, data| {
            if new_capacity.get() == indices.capacity() {
                return;
            }

            let elements_to_copy = min(indices.len, new_capacity.get());
            // Copy to memory all the elements that need to be preserved
            let elements = (0..elements_to_copy)
                .rev()
                .map(|offset| {
                    // These panics should never happen, because `elements_to_copy < indices.len`.
                    let idx = indices
                        .nth_element_from_end(offset)
                        .expect("element should be present");
                    data.get(idx).expect("element should be present")
                })
                .collect::<Vec<_>>();

            // clear the stable vector and fill with the elements
            data.clear();
            for element in elements {
                data.push(&element);
            }

            *indices = StableRingBufferIndices::new(new_capacity);
            indices.increase_len(elements_to_copy);
        });
    }

    /// Push new element to the buffer.
    ///
    /// Returns removed element if any
    pub fn push(&mut self, val: &T) -> Option<T> {
        self.with_indices_data_mut(|indices, data| {
            let new_index = indices.offset_to_index(indices.len());

            let replaced = if indices.len() == indices.capacity() {
                indices.increase_start(1);
                // This should never panic, because all indices < capacity are present in the data.
                Some(data.get(new_index).expect("element should be present"))
            } else {
                indices.increase_len(1);
                None
            };

            if new_index == data.len() {
                data.push(val);
            } else {
                // This should never panic, because `new_index` is inside the `data.len()`.
                data.set(new_index, val);
            }

            replaced
        })
    }

    /// Pop the last element from the buffer.
    pub fn pop(&mut self) -> Option<T> {
        self.with_indices_data_mut(|indices, data| {
            let new_len = indices.len.checked_sub(1)?;
            indices.decrease_len(1);
            let index = indices.offset_to_index(new_len);
            data.get(index)
        })
    }

    /// Remove `n` last elements from the buffer.
    pub fn truncate(&mut self, n: u64) {
        self.with_indices_data_mut(|indices, _| {
            indices.decrease_len(n);
        });
    }

    /// Get the first element if it exists.
    pub fn first(&self) -> Option<T> {
        self.nth_element(0)
    }

    /// Get the last element if it exists.
    pub fn last(&self) -> Option<T> {
        self.nth_element_from_end(0)
    }

    /// Get the `n`-th element from the start.
    pub fn nth_element(&self, n: u64) -> Option<T> {
        let index = self.indices.get().nth_element(n)?;
        self.data.get(index)
    }

    /// Get the `n`-th element from the end.
    pub fn nth_element_from_end(&self, n: u64) -> Option<T> {
        let index = self.indices.get().nth_element_from_end(n)?;
        self.data.get(index)
    }

    #[inline]
    fn with_indices_data_mut<R>(
        &mut self,
        f: impl Fn(&mut StableRingBufferIndices, &mut VecExt<T, DataMemory>) -> R,
    ) -> R {
        let mut indices = self.indices.get().clone();
        let result = f(&mut indices, &mut self.data);
        self.indices.set(indices);
        result
    }
}

#[cfg(test)]
mod tests {

    use std::fmt::Debug;

    use ic_stable_structures::VectorMemory;

    use super::*;

    /// Check the roundtrip value -> bytes -> value for `Storable` object
    fn test_storable_roundtrip<Val: Storable + Eq + std::fmt::Debug>(value: &Val) {
        let bytes = value.to_bytes();
        let decoded = Val::from_bytes(bytes);

        assert_eq!(&decoded, value);
    }

    #[test]
    fn test_indices_offset_to_index() {
        let indices = StableRingBufferIndices::new(4.try_into().unwrap());

        assert_eq!(0, indices.offset_to_index(0));
        assert_eq!(1, indices.offset_to_index(1));
        assert_eq!(2, indices.offset_to_index(2));
        assert_eq!(3, indices.offset_to_index(3));
        assert_eq!(0, indices.offset_to_index(4));
    }

    #[test]
    fn test_indices_nth_element() {
        let capacity = 4;
        let mut indices = StableRingBufferIndices::new(capacity.try_into().unwrap());

        for i in 0..10 {
            assert_eq!(None, indices.nth_element(i));
        }

        let len = 3;
        indices.increase_len(len);

        for i in 0..len {
            assert_eq!(Some(i), indices.nth_element(i));
        }
        for i in len..10 {
            assert_eq!(None, indices.nth_element(i));
        }

        let start = 2;
        indices.increase_start(2);

        for i in 0..len {
            assert_eq!(Some((start + i) % capacity), indices.nth_element(i));
        }
        for i in len..10 {
            assert_eq!(None, indices.nth_element(i));
        }
    }

    #[test]
    fn indices_should_be_storable() {
        test_storable_roundtrip(&StableRingBufferIndices::new(4.try_into().unwrap()));
        test_storable_roundtrip(&StableRingBufferIndices::new(4000.try_into().unwrap()));
    }

    fn check_buffer<T: Storable + Eq + Debug + Clone, DataMemory: Memory, IndicesMemory: Memory>(
        buffer: &StableRingBuffer<T, DataMemory, IndicesMemory>,
        expected: &[T],
    ) {
        assert_eq!(buffer.len(), expected.len() as u64);

        for (i, expected) in expected.iter().enumerate() {
            assert_eq!(Some(expected), buffer.nth_element(i as u64).as_ref());
        }

        assert_eq!(None, buffer.nth_element(expected.len() as _));
    }

    fn with_buffer(
        capacity: u64,
        f: impl Fn(&mut StableRingBuffer<u64, VectorMemory, VectorMemory>),
    ) {
        let mut buffer = StableRingBuffer::new(
            VectorMemory::default(),
            VectorMemory::default(),
            NonZeroU64::new(2).unwrap(),
        );
        buffer.clear();
        buffer.resize(capacity.try_into().unwrap());

        f(&mut buffer);
    }

    #[test]
    fn should_push() {
        with_buffer(3, |buffer| {
            check_buffer(buffer, &[]);
            assert!(buffer.is_empty());

            assert_eq!(buffer.push(&1), None);
            check_buffer(buffer, &[1]);

            assert_eq!(buffer.push(&2), None);
            check_buffer(buffer, &[1, 2]);

            assert_eq!(buffer.push(&3), None);
            check_buffer(buffer, &[1, 2, 3]);

            assert_eq!(buffer.push(&4), Some(1));
            check_buffer(buffer, &[2, 3, 4])
        });
    }

    #[test]
    fn should_pop() {
        with_buffer(5, |buffer| {
            check_buffer(buffer, &[]);

            // Checks for not-wrapped buffer.
            for i in 0..3 {
                assert_eq!(buffer.push(&i), None);
            }
            check_buffer(buffer, &[0, 1, 2]);

            assert_eq!(buffer.pop(), Some(2));
            check_buffer(buffer, &[0, 1]);

            assert_eq!(buffer.pop(), Some(1));
            check_buffer(buffer, &[0]);

            assert_eq!(buffer.push(&1), None);
            check_buffer(buffer, &[0, 1]);

            assert_eq!(buffer.pop(), Some(1));
            check_buffer(buffer, &[0]);

            assert_eq!(buffer.pop(), Some(0));
            check_buffer(buffer, &[]);

            assert_eq!(buffer.pop(), None);

            // Checks for wrapped buffer.
            for i in 0..5 {
                assert_eq!(buffer.push(&i), None);
            }
            assert_eq!(buffer.push(&5), Some(0));
            assert_eq!(buffer.push(&6), Some(1));

            let expected = vec![2, 3, 4, 5, 6];
            check_buffer(buffer, &expected);

            for i in 0..5 {
                check_buffer(buffer, &expected[..(5 - i)]);
                assert_eq!(buffer.pop(), Some(expected[4 - i]));
            }
            assert_eq!(buffer.pop(), None);
        });
    }

    #[test]
    fn should_truncate() {
        with_buffer(5, |buffer| {
            check_buffer(buffer, &[]);

            // Checks for not-wrapped buffer.
            for i in 0..5 {
                assert_eq!(buffer.push(&i), None);
            }
            check_buffer(buffer, &[0, 1, 2, 3, 4]);
            buffer.truncate(3);
            check_buffer(buffer, &[0, 1]);

            for i in 2..7 {
                buffer.push(&i);
            }
            check_buffer(buffer, &[2, 3, 4, 5, 6]);
            buffer.truncate(3);
            check_buffer(buffer, &[2, 3]);

            // Check replacement of truncated elements returns None.
            for i in 4..7 {
                assert_eq!(buffer.push(&i), None);
            }
            check_buffer(buffer, &[2, 3, 4, 5, 6]);
            assert_eq!(buffer.push(&7), Some(2));
            check_buffer(buffer, &[3, 4, 5, 6, 7]);
        });
    }

    #[test]
    fn should_resize_decrease() {
        with_buffer(3, |buffer| {
            let two = NonZeroU64::try_from(2).unwrap();
            let three = NonZeroU64::try_from(3).unwrap();

            // resize empty buffer
            buffer.resize(two);
            check_buffer(buffer, &[]);
            assert_eq!(two.get(), buffer.capacity());

            // resize smaller buffer
            buffer.resize(three);
            buffer.push(&1);
            buffer.resize(two);
            check_buffer(buffer, &[1]);
            assert_eq!(two.get(), buffer.capacity());

            // resize same size buffer
            buffer.clear();
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.resize(two);
            check_buffer(buffer, &[1, 2]);
            assert_eq!(two.get(), buffer.capacity());

            // resize bigger buffer
            buffer.clear();
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.push(&3);
            buffer.resize(two);
            check_buffer(buffer, &[2, 3]);
            assert_eq!(two.get(), buffer.capacity());

            // resize bigger buffer, rolled
            buffer.clear();
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.push(&3);
            buffer.push(&4);
            buffer.resize(two);
            check_buffer(buffer, &[3, 4]);
            assert_eq!(two.get(), buffer.capacity());
        });
    }

    #[test]
    fn test_resize_increase() {
        with_buffer(3, |buffer| {
            let three = NonZeroU64::try_from(3).unwrap();
            let four = NonZeroU64::try_from(4).unwrap();

            // resize empty buffer
            buffer.resize(four);
            check_buffer(buffer, &[]);
            assert_eq!(four.get(), buffer.capacity());

            // resize non-full buffer
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.resize(four);
            check_buffer(buffer, &[1, 2]);
            assert_eq!(four.get(), buffer.capacity());

            // resize full buffer
            buffer.clear();
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.push(&3);
            buffer.resize(four);
            check_buffer(buffer, &[1, 2, 3]);
            assert_eq!(four.get(), buffer.capacity());

            // resize full buffer rolled
            buffer.clear();
            buffer.resize(three);
            buffer.push(&1);
            buffer.push(&2);
            buffer.push(&3);
            buffer.push(&4);
            buffer.resize(four);
            buffer.push(&5);
            check_buffer(buffer, &[2, 3, 4, 5]);
            assert_eq!(four.get(), buffer.capacity());
        });
    }

    #[test]
    fn should_clear() {
        with_buffer(2, |buffer| {
            check_buffer(buffer, &[]);

            buffer.clear();
            check_buffer(buffer, &[]);
            assert_eq!(2, buffer.capacity());

            buffer.push(&1);
            buffer.push(&2);
            buffer.push(&3);
            check_buffer(buffer, &[2, 3]);
            assert_eq!(2, buffer.capacity());

            buffer.clear();
            check_buffer(buffer, &[]);
            assert_eq!(2, buffer.capacity());

            buffer.push(&1);
            check_buffer(buffer, &[1]);
            assert_eq!(2, buffer.capacity());

            buffer.clear();
            check_buffer(buffer, &[]);
            assert_eq!(2, buffer.capacity());
        })
    }
}
