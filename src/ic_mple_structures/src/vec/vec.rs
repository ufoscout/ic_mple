use ic_stable_structures::{Memory, Storable, Vec};

use crate::vec::VecStructure;


impl<T: Storable, M: Memory> VecStructure<T> for Option<Vec<T, M>> {
    fn is_empty(&self) -> bool {
        get_inner(self).is_empty()
    }

    fn clear(&mut self) {
        if let Some(vector) = self.take() {
            let memory = vector.into_memory();
            *self = Some(Vec::new(memory));
        }
    }

    fn len(&self) -> u64 {
        get_inner(self).len()
    }

    fn set(&mut self, index: u64, item: &T) {
        mut_inner(self).set(index, item)
    }

    fn get(&self, index: u64) -> Option<T> {
        get_inner(self).get(index)
    }

    fn push(&mut self, item: &T) {
        mut_inner(self).push(item);
    }

    fn pop(&mut self) -> Option<T> {
        mut_inner(self).pop()
    }
}

#[inline(always)]
    fn get_inner<T: Storable, M: Memory>(v: &Option<Vec<T, M>>) -> &Vec<T, M> {
        v.as_ref().expect("vector is always initialized")
    }

    #[inline(always)]
    fn mut_inner<T: Storable, M: Memory>(v: &mut Option<Vec<T, M>>) -> &mut Vec<T, M> {
        v.as_mut().expect("vector is always initialized")
    }