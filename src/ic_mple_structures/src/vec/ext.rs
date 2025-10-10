use ic_stable_structures::{Memory, Storable, vec};

use crate::vec::VecStructure;

pub struct VecExt<T: Storable, M: Memory>(Option<vec::Vec<T, M>>);

/// A stable analogue of the `std::vec::Vec`:
/// integer-indexed collection of mutable values that is able to grow.
impl<T: Storable, M: Memory> VecExt<T, M> {
    /// Initializes a vector in the specified memory.
    ///
    /// Complexity: O(1)
    ///
    /// PRECONDITION: the memory is either empty or contains a valid
    /// stable vector.
    pub fn init(memory: M) -> Self {
        Self(Some(vec::Vec::init(memory)))
    }

    /// Creates a new empty vector in the specified memory,
    /// overwriting any data structures the memory might have
    /// contained previously.
    pub fn new(memory: M) -> Self {
        Self(Some(vec::Vec::new(memory)))
    }

    /// Returns iterator over the elements in the vector
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.get_inner().iter()
    }

    #[inline(always)]
    fn mut_inner(&mut self) -> &mut vec::Vec<T, M> {
        self.0.as_mut().expect("vector is always initialized")
    }

    #[inline(always)]
    fn get_inner(&self) -> &vec::Vec<T, M> {
        self.0.as_ref().expect("vector is always initialized")
    }
}

impl<T: Storable, M: Memory> VecStructure<T> for VecExt<T, M> {
    fn is_empty(&self) -> bool {
        self.get_inner().is_empty()
    }

    fn clear(&mut self) {
        if let Some(vector) = self.0.take() {
            let memory = vector.into_memory();
            self.0 = Some(vec::Vec::new(memory));
        }
    }

    fn len(&self) -> u64 {
        self.get_inner().len()
    }

    fn set(&mut self, index: u64, item: &T) {
        self.mut_inner().set(index, item)
    }

    fn get(&self, index: u64) -> Option<T> {
        self.get_inner().get(index)
    }

    fn push(&mut self, item: &T) {
        self.mut_inner().push(item)
    }

    fn pop(&mut self) -> Option<T> {
        self.mut_inner().pop()
    }
}

#[cfg(test)]
mod tests {

    use ic_stable_structures::VectorMemory;

    use crate::test_utils::Array;

    use super::*;

    #[test]
    fn vec_works() {
        let mut vec = VecExt::<u64, _>::init(VectorMemory::default());

        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(0), None);

        vec.push(&1);
        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(1));
        assert_eq!(vec.get(1), None);

        vec.push(&2);
        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0), Some(1));
        assert_eq!(vec.get(1), Some(2));
        assert_eq!(vec.get(2), None);

        assert_eq!(vec.pop(), Some(2));
        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(0), Some(1));
        assert_eq!(vec.get(1), None);

        assert_eq!(vec.pop(), Some(1));
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(0), None);

        assert_eq!(vec.pop(), None);
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(0), None);

        vec.clear();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(0), None);

        vec.push(&1);
        vec.push(&2);
        let mut iter = vec.iter();
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(None, iter.next());
        drop(iter);

        vec.clear();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(0), None);
        assert_eq!(None, vec.iter().next());
    }

    #[should_panic]
    #[test]
    fn vec_unbounded_items() {
        let mut vec = VecExt::<String, _>::init(VectorMemory::default());

        let item = "I am an unbounded item".to_string();
        vec.push(&item);
        assert_eq!(Some(item), vec.get(0));
    }

    #[test]
    fn should_reuse_existing_data_on_init() {
        let memory = VectorMemory::default();

        {
            let mut log = VecExt::init(memory.clone());
            log.push(&Array([1u8, 1]))
        };

        {
            let log = VecExt::init(memory);
            assert!(!log.is_empty());
            assert_eq!(Some(Array([1u8, 1])), log.get(0));
        }
    }

    #[test]
    fn should_erase_existing_data_on_new() {
        let memory = VectorMemory::default();

        {
            let mut log = VecExt::new(memory.clone());
            log.push(&Array([1u8, 1]));
        };

        {
            let log = VecExt::<Array<2>, _>::new(memory);
            assert!(log.is_empty());
            assert_eq!(None, log.get(0));
        }
    }
}
