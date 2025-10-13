use std::borrow::Cow;

use ic_stable_structures::{Memory, StableCell};

use crate::{common::RefCodec, CellStructure};

/// A versioned stable cell.
pub struct VersionedStableCell<T: Clone, C: RefCodec<T>, M: Memory> {
    inner: StableCell<C, M>,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Clone, C: RefCodec<T>, M: Memory> VersionedStableCell<T, C, M> {

    /// Initializes a VersionedStableCell in the specified memory.
    ///
    /// PRECONDITION: the memory is either empty or contains a valid
    /// VersionedStableCell.
    pub fn init(memory: M, default_value: T) -> Self {
        Self{
            inner: StableCell::init(memory, C::encode(default_value)),
            phantom_t: std::marker::PhantomData
        }
    }

    /// Creates a new empty VersionedStableCell in the specified memory,
    /// overwriting any data structures the memory might have
    /// contained previously.
    pub fn new(memory: M, default_value: T) -> Self {
        Self{
            inner: StableCell::new(memory, C::encode(default_value)),
            phantom_t: std::marker::PhantomData
        }
    }
}

impl<T: Clone, C: RefCodec<T>, M: Memory> CellStructure<T> for VersionedStableCell<T, C, M> {

    fn get(&self) -> Cow<'_, T> {
        C::decode_ref(self.inner.get())
    }

    fn set(&mut self, value: T) {
        self.inner.set(C::encode(value));
    }

}

#[cfg(test)]
mod tests {
    use ic_stable_structures::VectorMemory;

    use crate::test_utils::{UserV1, UserV2, VersionedUser};

    use super::*;

    #[test]
    fn cell_should_use_user_codec() {
        // Arrange
        let memory = VectorMemory::default();

        // create cell with v1 data in it
        {
            let mut v1_cell = VersionedStableCell::<VersionedUser, VersionedUser,_>::init(memory.clone(), VersionedUser::V1(UserV1("test".to_string())));
            v1_cell.set(VersionedUser::V1(UserV1("test2".to_string())));
        }

        // create cell with v2 data that uses the codec
        let mut v2_cell = VersionedStableCell::<UserV2, VersionedUser,_>::init(memory, UserV2 {
            name: "test".to_string(),
            age: None
        });

        // Assert
        assert_eq!(
            v2_cell.get().as_ref(),
            &UserV2 {
                name: "test2".to_string(),
                age: None
            }
        );

        // Act
        v2_cell.set(UserV2 {
            name: "test3".to_string(),
            age: Some(42),
        });

        // Assert
        assert_eq!(
            v2_cell.get().as_ref(),
            &UserV2 {
                name: "test3".to_string(),
                age: Some(42)
            }
        );
    }

        #[test]
    fn should_reuse_existing_data_on_init() {
        let memory = VectorMemory::default();

        {
            let mut v2_cell = VersionedStableCell::<UserV2, VersionedUser,_>::init(memory.clone(), UserV2 {
            name: "test".to_string(),
            age: None
        });
                    v2_cell.set(UserV2 {
            name: "test3".to_string(),
            age: Some(42),
        });
        };

        {
            let v2_cell = VersionedStableCell::<UserV2, VersionedUser,_>::init(memory, UserV2 {
            name: "test".to_string(),
            age: None
        });
            assert_eq!(
                v2_cell.get().as_ref(),
                &UserV2 {
                    name: "test3".to_string(),
                    age: Some(42)
                }
            );
        }
    }

    #[test]
    fn should_erase_existing_data_on_new() {
        let memory = VectorMemory::default();

        {
            let mut v2_cell = VersionedStableCell::<UserV2, VersionedUser,_>::new(memory.clone(), UserV2 {
            name: "test".to_string(),
            age: None
        });
                    v2_cell.set(UserV2 {
            name: "test3".to_string(),
            age: Some(42),
        });
        };

        {
            let v2_cell = VersionedStableCell::<UserV2, VersionedUser,_>::new(memory, UserV2 {
            name: "test".to_string(),
            age: None
        });
            assert_eq!(
                v2_cell.get().as_ref(),
                &UserV2 {
                    name: "test".to_string(),
                    age: None
                }
            );
        }
    }
}
