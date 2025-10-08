use std::borrow::Cow;

use ic_stable_structures::{Memory, StableCell, Storable};

use crate::{cell::CellStructure, common::codec::Codec};

/// A versioned stable cell.
pub struct VersionedStableCell<T: Storable, M: Memory, C: Codec<T, D>, D: Clone> {
    cell: StableCell<T, M>,
    codec: C,
    phantom_d: std::marker::PhantomData<D>,
}

impl<T: Storable, M: Memory, C: Codec<T, D>, D: Clone> VersionedStableCell<T, M, C, D> {

    pub fn new(cell: StableCell<T, M>, codec: C) -> Self {
        Self {
            cell,
            codec,
            phantom_d: std::marker::PhantomData,
        }
    }

}

impl<T: Storable, M: Memory, C: Codec<T, D>, D: Clone> CellStructure<D> for VersionedStableCell<T, M, C, D> {

    fn get(&self) -> Cow<'_, D> {
        self.codec.decode_ref(self.cell.get())        
    }

    fn set(&mut self, value: D) {
        self.cell.set(self.codec.encode(value));
    }

}

#[cfg(test)]
mod tests {
    use ic_stable_structures::{memory_manager::{MemoryId, MemoryManager}, DefaultMemoryImpl};

    use crate::test_utils::{UserCodec, UserV1, UserV2, VersionedUser};

    use super::*;
    
    #[test]
    fn cell_should_use_user_codec() {
        // Arrange
        let memory = MemoryManager::init(DefaultMemoryImpl::default()).get(MemoryId::new(1));
        let mut cell = StableCell::new(memory, VersionedUser::V1(UserV1("test".to_string())));
        cell.set(VersionedUser::V1(UserV1("test2".to_string())));

        let mut versioned_cell = VersionedStableCell::new(cell, UserCodec);

        // Assert
        assert_eq!(versioned_cell.get().as_ref(), &UserV2 {
            name: "test2".to_string(),
            age: None
        });
        
        // Act
        versioned_cell.set(UserV2 {
            name: "test3".to_string(),
            age: Some(42)
        });

        // Assert
        assert_eq!(versioned_cell.get().as_ref(), &UserV2 {
            name: "test3".to_string(),
            age: Some(42)
        });

    }

}