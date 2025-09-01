use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PermissionError {
    #[error("NotAuthorized")]
    NotAuthorized,

    #[error("AnonimousUserNotAllowed")]
    AnonimousUserNotAllowed,
}
