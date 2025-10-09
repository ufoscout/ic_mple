use std::borrow::Cow;

use ic_stable_structures::{Memory, StableCell, Storable};

use crate::{cell::CellStructure, common::RefCodec};

/// A versioned stable cell.
pub struct VersionedStableCell<T: Storable, M: Memory, C: RefCodec<T, D>, D: Clone> {
    inner: StableCell<T, M>,
    codec: C,
    phantom_d: std::marker::PhantomData<D>,
}

impl<T: Storable, M: Memory, C: RefCodec<T, D>, D: Clone> VersionedStableCell<T, M, C, D> {
    pub fn new(cell: StableCell<T, M>, codec: C) -> Self {
        Self {
            inner: cell,
            codec,
            phantom_d: std::marker::PhantomData,
        }
    }
}

impl<T: Storable, M: Memory, C: RefCodec<T, D>, D: Clone> CellStructure<D>
    for VersionedStableCell<T, M, C, D>
{
    fn get(&self) -> Cow<'_, D> {
        self.codec.decode_ref(self.inner.get())
    }

    fn set(&mut self, value: D) {
        self.inner.set(self.codec.encode(value));
    }
}

#[cfg(test)]
mod tests {
    use ic_stable_structures::VectorMemory;

    use crate::test_utils::{UserCodec, UserV1, UserV2, VersionedUser};

    use super::*;

    #[test]
    fn cell_should_use_user_codec() {
        // Arrange
        let mut cell = StableCell::new(
            VectorMemory::default(),
            VersionedUser::V1(UserV1("test".to_string())),
        );
        cell.set(VersionedUser::V1(UserV1("test2".to_string())));

        let mut versioned_cell = VersionedStableCell::new(cell, UserCodec);

        // Assert
        assert_eq!(
            versioned_cell.get().as_ref(),
            &UserV2 {
                name: "test2".to_string(),
                age: None
            }
        );

        // Act
        versioned_cell.set(UserV2 {
            name: "test3".to_string(),
            age: Some(42),
        });

        // Assert
        assert_eq!(
            versioned_cell.get().as_ref(),
            &UserV2 {
                name: "test3".to_string(),
                age: Some(42)
            }
        );
    }
}
