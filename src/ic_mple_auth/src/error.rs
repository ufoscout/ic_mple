use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("NotAuthorized")]
    NotAuthorized,

    #[error("AnonimousUserNotAllowed")]
    AnonimousUserNotAllowed,
}
