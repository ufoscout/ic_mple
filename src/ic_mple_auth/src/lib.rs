use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;

use candid::CandidType;
use candid::Decode;
use candid::Encode;
use candid::Principal;
use ic_mple_utils::store::Storage;
use ic_stable_structures::BTreeMap;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::Storable;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use log::info;
use serde::de::DeserializeOwned;

use crate::error::PermissionError;

pub mod error;

#[derive(Debug, CandidType, PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone)]
pub struct PermissionList<
    T: PartialEq + CandidType + PartialEq + Eq + serde::Serialize + Hash + Clone + std::fmt::Debug,
> {
    pub permissions: HashSet<T>,
}

impl<T: PartialEq + CandidType + PartialEq + Eq + serde::Serialize + Hash + Clone + std::fmt::Debug>
    Default for PermissionList<T>
{
    fn default() -> Self {
        Self {
            permissions: Default::default(),
        }
    }
}

impl<T: PartialEq + CandidType + PartialEq + Eq + serde::Serialize + Hash + Clone + std::fmt::Debug>
    Storable for PermissionList<T>
where
    T: DeserializeOwned,
{
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Cow::from(Encode!(&self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }
}

pub type AuthServiceStorage<T> =
    BTreeMap<Principal, PermissionList<T>, VirtualMemory<DefaultMemoryImpl>>;

/// A service for managing user permissions
pub struct AuthService<
    S: Storage<AuthServiceStorage<T>>,
    T: PartialEq + CandidType + PartialEq + Eq + serde::Serialize + Hash + Clone + std::fmt::Debug,
> where
    T: DeserializeOwned,
{
    permission_storage: S,
    phantom: std::marker::PhantomData<T>,
}

impl<
    S: Storage<AuthServiceStorage<T>>,
    T: PartialEq + CandidType + PartialEq + Eq + serde::Serialize + Hash + Clone + std::fmt::Debug,
> AuthService<S, T>
where
    T: DeserializeOwned,
{
    /// Instantiates a new PermissionService
    pub fn new(permission_storage: S) -> Self {
        Self {
            permission_storage,
            phantom: std::marker::PhantomData,
        }
    }

    /// Panics if the user does not have the required permission
    pub fn must_have_permission(&self, principal: &Principal, permission: T) {
        self.check_has_permission(principal, permission).unwrap();
    }

    /// Panics if the user does not have all the required permissions
    pub fn must_have_all_permissions(&self, principal: &Principal, permissions: &[T]) {
        self.check_has_all_permissions(principal, permissions)
            .unwrap();
    }

    /// Panics if the user does not have at least one of the required permissions
    pub fn must_have_any_permission(&self, principal: &Principal, permissions: &[T]) {
        self.check_has_any_permission(principal, permissions)
            .unwrap();
    }

    /// Returns NotAuthorized error if the user does not have the required permission
    pub fn check_has_permission(
        &self,
        principal: &Principal,
        permission: T,
    ) -> Result<(), PermissionError> {
        if self.has_all_permissions(principal, &[permission]) {
            Ok(())
        } else {
            Err(PermissionError::NotAuthorized)
        }
    }

    /// Returns NotAuthorized error if the user does not have all permissions
    pub fn check_has_all_permissions(
        &self,
        principal: &Principal,
        permissions: &[T],
    ) -> Result<(), PermissionError> {
        if self.has_all_permissions(principal, permissions) {
            Ok(())
        } else {
            Err(PermissionError::NotAuthorized)
        }
    }

    /// Returns whether the user has all the required permissions
    pub fn has_all_permissions(&self, principal: &Principal, permissions: &[T]) -> bool {
        self.permission_storage.with_borrow(|permission_storage| {
            if let Some(permissions_list) = permission_storage.get(principal) {
                permissions
                    .iter()
                    .all(|item| permissions_list.permissions.contains(item))
            } else {
                permissions.is_empty()
            }
        })
    }

    /// Returns NotAuthorized error if the user does not have at least one of the permissions
    pub fn check_has_any_permission(
        &self,
        principal: &Principal,
        permissions: &[T],
    ) -> Result<(), PermissionError> {
        if self.has_any_permission(principal, permissions) {
            Ok(())
        } else {
            Err(PermissionError::NotAuthorized)
        }
    }

    /// Return whether the user has at least one of the required permissions
    pub fn has_any_permission(&self, principal: &Principal, permissions: &[T]) -> bool {
        self.permission_storage.with_borrow(|permission_storage| {
            if let Some(permissions_list) = permission_storage.get(principal) {
                permissions
                    .iter()
                    .any(|item| permissions_list.permissions.contains(item))
                    || permissions.is_empty()
            } else {
                permissions.is_empty()
            }
        })
    }

    /// Add permissions to a user
    pub fn add_permissions(
        &mut self,
        principal: Principal,
        permissions: Vec<T>,
    ) -> Result<PermissionList<T>, PermissionError> {
        self.check_anonymous_principal(&principal)?;
        self.permission_storage
            .with_borrow_mut(|permission_storage| {
                info!(
                    "Adding permissions {:?} to principal {}",
                    permissions, principal
                );

                let mut existing_permissions =
                    permission_storage.get(&principal).unwrap_or_default();
                for permission in permissions {
                    existing_permissions.permissions.insert(permission);
                }
                permission_storage.insert(principal, existing_permissions.clone());
                Ok(existing_permissions)
            })
    }

    /// Remove permissions from a user
    pub fn remove_permissions(
        &mut self,
        principal: Principal,
        permissions: &[T],
    ) -> Result<PermissionList<T>, PermissionError> {
        self.check_anonymous_principal(&principal)?;
        self.permission_storage
            .with_borrow_mut(|permission_storage| {
                let mut existing_permissions =
                    permission_storage.get(&principal).unwrap_or_default();

                info!(
                    "Removing permissions {:?} from principal {principal}",
                    permissions
                );

                existing_permissions
                    .permissions
                    .retain(|x| !permissions.contains(x));
                if !existing_permissions.permissions.is_empty() {
                    permission_storage.insert(principal, existing_permissions.clone());
                } else {
                    permission_storage.remove(&principal);
                }
                Ok(existing_permissions)
            })
    }

    /// Return the user permissions
    pub fn get_permissions(&self, principal: &Principal) -> PermissionList<T> {
        self.permission_storage
            .with_borrow(|permission_storage| permission_storage.get(principal).unwrap_or_default())
    }

    /// Clear the Whitelist state
    pub fn clear(&mut self) {
        self.permission_storage
            .with_borrow_mut(|permission_storage| permission_storage.clear_new())
    }

    fn check_anonymous_principal(&self, principal: &Principal) -> Result<(), PermissionError> {
        if principal == &Principal::anonymous() {
            return Err(PermissionError::AnonimousUserNotAllowed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, collections::HashSet};

    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use serde::Deserialize;

    use super::*;

    #[test]
    fn test_candid_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![TestPermission::Admin, TestPermission::ReadLogs]),
        };

        let serialized = Encode!(&permission_list).unwrap();
        let deserialized = Decode!(serialized.as_slice(), PermissionList<TestPermission>).unwrap();

        assert_eq!(permission_list, deserialized);
    }

    #[test]
    fn test_storable_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![TestPermission::Admin, TestPermission::ReadLogs]),
        };

        let serialized = permission_list.to_bytes();
        let deserialized = PermissionList::from_bytes(serialized);

        assert_eq!(permission_list, deserialized);
    }

    #[test]
    fn should_have_no_permissions() {
        // Arrange
        let mut permissions = new_permission_service();
        let principal = Principal::from_slice(&[1; 29]);

        // Assert
        assert!(permissions.has_all_permissions(&principal, &[]));
        assert!(!permissions.has_all_permissions(&principal, &[TestPermission::ReadLogs]));
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(!permissions.has_any_permission(&principal, &[TestPermission::UpdateLogs]));

        permissions
            .add_permissions(principal, vec![TestPermission::ReadLogs])
            .unwrap();

        assert!(permissions.has_all_permissions(&principal, &[]));
        assert!(!permissions.has_all_permissions(&principal, &[TestPermission::UpdateLogs]));
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(!permissions.has_any_permission(&principal, &[TestPermission::UpdateLogs]));
    }

    #[test]
    fn should_return_the_user_permissions() {
        // Arrange
        let mut permissions = new_permission_service();
        permissions.clear();

        let principal = Principal::from_slice(&[1; 29]);

        // Assert
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![TestPermission::ReadLogs])
            },
            permissions
                .add_permissions(principal, vec![TestPermission::ReadLogs])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![TestPermission::ReadLogs])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![TestPermission::ReadLogs])
            },
            permissions
                .add_permissions(
                    principal,
                    vec![TestPermission::ReadLogs, TestPermission::ReadLogs]
                )
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![TestPermission::ReadLogs])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![
                    TestPermission::ReadLogs,
                    TestPermission::UpdateLogs
                ])
            },
            permissions
                .add_permissions(principal, vec![TestPermission::UpdateLogs])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![
                    TestPermission::ReadLogs,
                    TestPermission::UpdateLogs
                ])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions
                .remove_permissions(
                    principal,
                    &[
                        TestPermission::UpdateLogs,
                        TestPermission::ReadLogs,
                        TestPermission::Admin
                    ]
                )
                .unwrap()
        );
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions
                .remove_permissions(
                    principal,
                    &[TestPermission::UpdateLogs, TestPermission::ReadLogs]
                )
                .unwrap()
        );
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );
    }

    #[test]
    fn should_add_and_remove_permissions() {
        // Arrange
        let mut permissions = new_permission_service();
        let principal_1 = Principal::from_slice(&[1; 29]);
        let principal_2 = Principal::from_slice(&[2; 29]);
        let principal_3 = Principal::from_slice(&[3; 29]);
        let principal_4 = Principal::from_slice(&[4; 29]);
        let principal_5 = Principal::from_slice(&[5; 29]);

        // Add permissions
        {
            permissions
                .add_permissions(principal_2, vec![TestPermission::ReadLogs])
                .unwrap();
            permissions
                .add_permissions(principal_3, vec![TestPermission::UpdateLogs])
                .unwrap();
            permissions
                .add_permissions(
                    principal_4,
                    vec![TestPermission::ReadLogs, TestPermission::UpdateLogs],
                )
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![TestPermission::ReadLogs])
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![TestPermission::UpdateLogs])
                .unwrap();

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_all_permissions(&principal_1, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_any_permission(&principal_1, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(permissions.has_all_permissions(&principal_2, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_all_permissions(&principal_2, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(permissions.has_any_permission(&principal_2, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_any_permission(&principal_2, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_2,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[TestPermission::ReadLogs]));
            assert!(permissions.has_all_permissions(&principal_3, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[TestPermission::ReadLogs]));
            assert!(permissions.has_any_permission(&principal_3, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_3,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(permissions.has_all_permissions(&principal_4, &[TestPermission::ReadLogs]));
            assert!(permissions.has_all_permissions(&principal_4, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_all_permissions(
                &principal_4,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(permissions.has_any_permission(&principal_4, &[TestPermission::ReadLogs]));
            assert!(permissions.has_any_permission(&principal_4, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_4,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(permissions.has_all_permissions(&principal_5, &[TestPermission::ReadLogs]));
            assert!(permissions.has_all_permissions(&principal_5, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_all_permissions(
                &principal_5,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(permissions.has_any_permission(&principal_5, &[TestPermission::ReadLogs]));
            assert!(permissions.has_any_permission(&principal_5, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_5,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
        }

        // remove permissions
        {
            permissions
                .remove_permissions(principal_1, &[TestPermission::ReadLogs])
                .unwrap();
            permissions
                .remove_permissions(principal_2, &[TestPermission::ReadLogs])
                .unwrap();
            permissions
                .remove_permissions(principal_3, &[TestPermission::ReadLogs])
                .unwrap();
            permissions
                .remove_permissions(principal_4, &[TestPermission::ReadLogs])
                .unwrap();
            permissions
                .remove_permissions(
                    principal_5,
                    &[TestPermission::ReadLogs, TestPermission::UpdateLogs],
                )
                .unwrap();

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_all_permissions(&principal_1, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_any_permission(&principal_1, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(!permissions.has_all_permissions(&principal_2, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_all_permissions(&principal_2, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_2, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_any_permission(&principal_2, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_any_permission(
                &principal_2,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[TestPermission::ReadLogs]));
            assert!(permissions.has_all_permissions(&principal_3, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[TestPermission::ReadLogs]));
            assert!(permissions.has_any_permission(&principal_3, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_3,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(!permissions.has_all_permissions(&principal_4, &[TestPermission::ReadLogs]));
            assert!(permissions.has_all_permissions(&principal_4, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_4,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_4, &[TestPermission::ReadLogs]));
            assert!(permissions.has_any_permission(&principal_4, &[TestPermission::UpdateLogs]));
            assert!(permissions.has_any_permission(
                &principal_4,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));

            assert!(!permissions.has_all_permissions(&principal_5, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_all_permissions(&principal_5, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_all_permissions(
                &principal_5,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
            assert!(!permissions.has_any_permission(&principal_5, &[TestPermission::ReadLogs]));
            assert!(!permissions.has_any_permission(&principal_5, &[TestPermission::UpdateLogs]));
            assert!(!permissions.has_any_permission(
                &principal_5,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            ));
        }
    }

    #[test]
    fn should_check_permissions_and_return_error() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![TestPermission::ReadLogs])
            .unwrap();

        // Assert
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_has_permission(&principal_1, TestPermission::UpdateLogs)
        );
        assert!(
            permissions
                .check_has_permission(&principal_1, TestPermission::ReadLogs)
                .is_ok()
        );
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_has_all_permissions(
                &principal_1,
                &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
            )
        );
        assert!(
            permissions
                .check_has_all_permissions(&principal_1, &[TestPermission::ReadLogs])
                .is_ok()
        );
        assert!(
            permissions
                .check_has_all_permissions(&principal_1, &[TestPermission::UpdateLogs])
                .is_err()
        );

        assert!(
            permissions
                .check_has_any_permission(
                    &principal_1,
                    &[TestPermission::ReadLogs, TestPermission::UpdateLogs]
                )
                .is_ok()
        );
        assert!(
            permissions
                .check_has_any_permission(&principal_1, &[TestPermission::ReadLogs])
                .is_ok()
        );
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_has_any_permission(&principal_1, &[TestPermission::UpdateLogs])
        );
    }

    #[test]
    fn check_cannot_add_permissions_for_anonymous_principal() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::anonymous();

        let res = permissions
            .add_permissions(principal_1, vec![TestPermission::ReadLogs])
            .unwrap_err();

        assert_eq!(PermissionError::AnonimousUserNotAllowed, res);
    }

    #[test]
    fn check_cannot_remove_permissions_for_anonymous_principal() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::anonymous();

        let res = permissions
            .remove_permissions(principal_1, &[TestPermission::ReadLogs])
            .unwrap_err();

        assert_eq!(PermissionError::AnonimousUserNotAllowed, res);
    }

    /// Test that panic happens when the user does not have the permission
    #[test]
    #[should_panic(expected = "NotAuthorized")]
    fn should_panic_if_user_does_not_have_permission() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![TestPermission::ReadLogs])
            .unwrap();

        // Act
        permissions.must_have_permission(&principal_1, TestPermission::Admin);
    }

    /// Test that panic does not happen when the user haves the permission
    #[test]
    fn should_not_panic_if_user_has_permission() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![TestPermission::ReadLogs])
            .unwrap();

        // Act
        permissions.must_have_permission(&principal_1, TestPermission::ReadLogs);
    }

    /// Test that panic happens when the user does not have all the permissions
    #[test]
    #[should_panic(expected = "NotAuthorized")]
    fn should_panic_if_user_does_not_have_all_permissions() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(
                principal_1,
                vec![TestPermission::ReadLogs, TestPermission::UpdateLogs],
            )
            .unwrap();

        // Act
        permissions.must_have_all_permissions(
            &principal_1,
            &[
                TestPermission::ReadLogs,
                TestPermission::UpdateLogs,
                TestPermission::Admin,
            ],
        );
    }

    /// Test that panic does not happen when the user has all the permissions
    #[test]
    fn should_not_panic_if_user_has_all_permissions() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(
                principal_1,
                vec![TestPermission::ReadLogs, TestPermission::UpdateLogs],
            )
            .unwrap();

        // Act
        permissions.must_have_all_permissions(
            &principal_1,
            &[TestPermission::ReadLogs, TestPermission::UpdateLogs],
        );
    }

    /// Test that panic happens when the user does not have any of the permissions
    #[test]
    #[should_panic(expected = "NotAuthorized")]
    fn should_panic_if_user_does_not_have_any_permissions() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(
                principal_1,
                vec![TestPermission::ReadLogs, TestPermission::UpdateLogs],
            )
            .unwrap();

        // Act
        permissions.must_have_any_permission(&principal_1, &[TestPermission::Admin]);
    }

    /// Test that panic does not happen when the user has any of the permissions
    #[test]
    fn should_not_panic_if_user_has_any_permissions() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![TestPermission::ReadLogs])
            .unwrap();

        // Act
        permissions.must_have_any_permission(
            &principal_1,
            &[
                TestPermission::ReadLogs,
                TestPermission::UpdateLogs,
                TestPermission::Admin,
            ],
        );
    }

    fn new_permission_service() -> TestPermissionService {
        let store = RefCell::new(BTreeMap::new(
            MemoryManager::init(DefaultMemoryImpl::default()).get(MemoryId::new(1)),
        ));
        AuthService::new(store)
    }

    type TestPermissionService =
        AuthService<RefCell<AuthServiceStorage<TestPermission>>, TestPermission>;

    /// Principal specific permission
    #[derive(
        Debug,
        Clone,
        CandidType,
        Deserialize,
        Hash,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        serde::Serialize,
    )]
    enum TestPermission {
        Admin,
        ReadLogs,
        ResetState,
        UpdateLogs,
    }
}
