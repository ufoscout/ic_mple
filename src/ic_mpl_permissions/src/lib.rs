use std::borrow::Cow;
use std::collections::HashSet;

use candid::CandidType;
use candid::Decode;
use candid::Deserialize;
use candid::Encode;
use candid::Principal;
use ic_mple_utils::store::Storage;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::BTreeMap;
use ic_stable_structures::Storable;
use log::info;

use crate::error::PermissionError;

pub mod error;

/// Principal specific permission
#[derive(
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
)]
pub enum Permission {
    /// Gives administrator permissions
    Admin,
    /// Allows calling the endpoints to read the logs and get runtime statistics
    ReadLogs,
    /// Allows caller to reset the EVM state
    ResetEvmState,
    /// Allows calling the endpoints to set the logs configuration
    UpdateLogsConfiguration,
    /// Allows calling the endpoints to set validate unsafe blocks
    ValidateUnsafeBlocks,
    /// Allows the signature verification canister to send transaction to
    /// the EVM Canister
    PrivilegedSendTransaction,
}

#[derive(Debug, Clone, Default, CandidType, Deserialize, PartialEq, Eq, serde::Serialize)]
pub struct PermissionList {
    pub permissions: HashSet<Permission>,
}

impl Storable for PermissionList {
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

pub type PermissionServiceStorage = BTreeMap<Principal, PermissionList, VirtualMemory<DefaultMemoryImpl>>;

/// A service for managing user permissions
pub struct PermissionService<S: Storage<PermissionServiceStorage>> {
    permission_data: S,
}

impl<S: Storage<PermissionServiceStorage>> PermissionService<S> {

    /// Instantiates a new PermissionService
    pub fn new(permission_data: S) -> Self {
        Self {
            permission_data,
        }
    }

    /// Checks if the user has the Admin permission
    pub fn check_admin(&self, principal: &Principal) -> Result<(), PermissionError> {
        self.check_has_all_permissions(principal, &[Permission::Admin])
    }

    /// Returns NotAuthorized error if the user does not have all permissions
    pub fn check_has_all_permissions(
        &self,
        principal: &Principal,
        permissions: &[Permission],
    ) -> Result<(), PermissionError> {
        if self.has_all_permissions(principal, permissions) {
            Ok(())
        } else {
            Err(PermissionError::NotAuthorized)
        }
    }

    /// Returns whether the user has all the required permissions
    pub fn has_all_permissions(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        self.permission_data.with_borrow(|permission_data| {
            if let Some(permissions_list) = permission_data.get(principal) {
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
        permissions: &[Permission],
    ) -> Result<(), PermissionError> {
        if self.has_any_permission(principal, permissions) {
            Ok(())
        } else {
            Err(PermissionError::NotAuthorized)
        }
    }

    /// Return whether the user has at least one of the required permissions
    pub fn has_any_permission(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        self.permission_data.with_borrow(|permission_data| {

        if let Some(permissions_list) = permission_data.get(principal) {
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
        permissions: Vec<Permission>,
    ) -> Result<PermissionList, PermissionError> {
        self.check_anonymous_principal(&principal)?;
        self.permission_data.with_borrow_mut(|permission_data| {
            info!(
                "Adding permissions {:?} to principal {}",
                permissions, principal
            );

            let mut existing_permissions = permission_data.get(&principal).unwrap_or_default();
            for permission in permissions {
                existing_permissions.permissions.insert(permission);
            }
            permission_data
                .insert(principal, existing_permissions.clone());
            Ok(existing_permissions)
        })
    }

    /// Remove permissions from a user
    pub fn remove_permissions(
        &mut self,
        principal: Principal,
        permissions: &[Permission],
    ) -> Result<PermissionList, PermissionError> {
        self.check_anonymous_principal(&principal)?;
        self.permission_data.with_borrow_mut(|permission_data| {
            let mut existing_permissions = permission_data.get(&principal).unwrap_or_default();

            info!(
                "Removing permissions {:?} from principal {principal}",
                permissions
            );

            existing_permissions
                .permissions
                .retain(|x| !permissions.contains(x));
            if !existing_permissions.permissions.is_empty() {
                permission_data
                    .insert(principal, existing_permissions.clone());
            } else {
                permission_data.remove(&principal);
            }
            Ok(existing_permissions)
        })
    }

    /// Return the user permissions
    pub fn get_permissions(&self, principal: &Principal) -> PermissionList {
        self.permission_data.with_borrow(|permission_data| {
            permission_data.get(principal).unwrap_or_default()
        })
    }

    /// Clear the Whitelist state
    pub fn clear(&mut self) {
        self.permission_data.with_borrow_mut(|permission_data| {
            permission_data.clear_new()
        })
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

    use super::*;

        #[test]
    fn test_candid_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![Permission::Admin, Permission::ReadLogs]),
        };

        let serialized = Encode!(&permission_list).unwrap();
        let deserialized = Decode!(serialized.as_slice(), PermissionList).unwrap();

        assert_eq!(permission_list, deserialized);
    }

    #[test]
    fn test_storable_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![Permission::Admin, Permission::ReadLogs]),
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
        assert!(!permissions.has_all_permissions(&principal, &[Permission::ReadLogs]));
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(
            !permissions.has_any_permission(&principal, &[Permission::UpdateLogsConfiguration])
        );

        permissions
            .add_permissions(principal, vec![Permission::ReadLogs])
            .unwrap();

        assert!(permissions.has_all_permissions(&principal, &[]));
        assert!(
            !permissions.has_all_permissions(&principal, &[Permission::UpdateLogsConfiguration])
        );
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(
            !permissions.has_any_permission(&principal, &[Permission::UpdateLogsConfiguration])
        );
    }

    #[test]
    fn should_return_the_user_permissions() {
        // Arrange
let mut permissions = new_permission_service();        permissions.clear();

        let principal = Principal::from_slice(&[1; 29]);

        // Assert
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::ReadLogs])
            },
            permissions
                .add_permissions(principal, vec![Permission::ReadLogs])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::ReadLogs])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::ReadLogs])
            },
            permissions
                .add_permissions(principal, vec![Permission::ReadLogs, Permission::ReadLogs])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::ReadLogs])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![
                    Permission::ReadLogs,
                    Permission::UpdateLogsConfiguration
                ])
            },
            permissions
                .add_permissions(principal, vec![Permission::UpdateLogsConfiguration])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![
                    Permission::ReadLogs,
                    Permission::UpdateLogsConfiguration
                ])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions.remove_permissions(
                principal,
                &[
                    Permission::UpdateLogsConfiguration,
                    Permission::ReadLogs,
                    Permission::Admin
                ]
            ).unwrap()
        );
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions.remove_permissions(
                principal,
                &[Permission::UpdateLogsConfiguration, Permission::ReadLogs]
            ).unwrap()
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
                .add_permissions(principal_2, vec![Permission::ReadLogs])
                .unwrap();
            permissions
                .add_permissions(principal_3, vec![Permission::UpdateLogsConfiguration])
                .unwrap();
            permissions
                .add_permissions(
                    principal_4,
                    vec![Permission::ReadLogs, Permission::UpdateLogsConfiguration],
                )
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![Permission::ReadLogs])
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![Permission::UpdateLogsConfiguration])
                .unwrap();

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_all_permissions(&principal_1, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_any_permission(&principal_1, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(permissions.has_all_permissions(&principal_2, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_all_permissions(&principal_2, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(permissions.has_any_permission(&principal_2, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_any_permission(&principal_2, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_2,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_all_permissions(&principal_3, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_any_permission(&principal_3, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_3,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(permissions.has_all_permissions(&principal_4, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_all_permissions(&principal_4, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_all_permissions(
                &principal_4,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(permissions.has_any_permission(&principal_4, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_any_permission(&principal_4, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_4,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(permissions.has_all_permissions(&principal_5, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_all_permissions(&principal_5, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_all_permissions(
                &principal_5,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(permissions.has_any_permission(&principal_5, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_any_permission(&principal_5, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_5,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
        }

        // remove permissions
        {
            permissions.remove_permissions(principal_1, &[Permission::ReadLogs]);
            permissions.remove_permissions(principal_2, &[Permission::ReadLogs]);
            permissions.remove_permissions(principal_3, &[Permission::ReadLogs]);
            permissions.remove_permissions(principal_4, &[Permission::ReadLogs]);
            permissions.remove_permissions(
                principal_5,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration],
            );

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_all_permissions(&principal_1, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_any_permission(&principal_1, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(!permissions.has_all_permissions(&principal_2, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_all_permissions(&principal_2, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_2, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_any_permission(&principal_2, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_any_permission(
                &principal_2,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_all_permissions(&principal_3, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_any_permission(&principal_3, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_3,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(!permissions.has_all_permissions(&principal_4, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_all_permissions(&principal_4, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_4,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_4, &[Permission::ReadLogs]));
            assert!(
                permissions
                    .has_any_permission(&principal_4, &[Permission::UpdateLogsConfiguration])
            );
            assert!(permissions.has_any_permission(
                &principal_4,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));

            assert!(!permissions.has_all_permissions(&principal_5, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_all_permissions(&principal_5, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_all_permissions(
                &principal_5,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
            assert!(!permissions.has_any_permission(&principal_5, &[Permission::ReadLogs]));
            assert!(
                !permissions
                    .has_any_permission(&principal_5, &[Permission::UpdateLogsConfiguration])
            );
            assert!(!permissions.has_any_permission(
                &principal_5,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            ));
        }
    }

    #[test]
    fn should_check_permissions_and_return_error() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![Permission::ReadLogs])
            .unwrap();

        // Assert
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_has_all_permissions(
                &principal_1,
                &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
            )
        );
        assert!(
            permissions
                .check_has_all_permissions(&principal_1, &[Permission::ReadLogs])
                .is_ok()
        );
        assert!(
            permissions
                .check_has_all_permissions(&principal_1, &[Permission::UpdateLogsConfiguration])
                .is_err()
        );

        assert!(
            permissions
                .check_has_any_permission(
                    &principal_1,
                    &[Permission::ReadLogs, Permission::UpdateLogsConfiguration]
                )
                .is_ok()
        );
        assert!(
            permissions
                .check_has_any_permission(&principal_1, &[Permission::ReadLogs])
                .is_ok()
        );
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions
                .check_has_any_permission(&principal_1, &[Permission::UpdateLogsConfiguration])
        );
    }

    #[test]
    fn should_check_if_user_is_admin() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::from_slice(&[1; 29]);
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );

        permissions
            .add_permissions(principal_1, vec![Permission::ReadLogs])
            .unwrap();
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );

        permissions
            .add_permissions(principal_1, vec![Permission::Admin])
            .unwrap();
        assert_eq!(Ok(()), permissions.check_admin(&principal_1));

        permissions.remove_permissions(principal_1, &[Permission::Admin]);
        assert_eq!(
            Err(PermissionError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );
    }

    #[test]
    fn check_anonymous_principal_is_rejected() {
        // Arrange
        let mut permissions = new_permission_service();

        let principal_1 = Principal::anonymous();

        let res = permissions
            .add_permissions(principal_1, vec![Permission::ReadLogs])
            .unwrap_err();

        assert_eq!(
            PermissionError::AnonimousUserNotAllowed,
            res
        );
    }

    fn new_permission_service() -> PermissionService<RefCell<PermissionServiceStorage>> {
        let store = RefCell::new(BTreeMap::new(
            MemoryManager::init(DefaultMemoryImpl::default()).get(MemoryId::new(1)),
        ));
        PermissionService::new(store)
    }
}
