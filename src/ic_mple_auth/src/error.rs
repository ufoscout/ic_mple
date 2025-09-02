use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, CandidType, Serialize, Deserialize, Clone)]
pub enum AuthError {
    #[error("NotAuthorized")]
    NotAuthorized,

    #[error("AnonimousUserNotAllowed")]
    AnonimousUserNotAllowed,
}
