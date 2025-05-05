use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanisterClientError {
    #[error("canister call failed: {0:?}")]
    CanisterError(IcError),

    #[error(transparent)]
    CandidError(#[from] candid::Error),

    #[cfg(feature = "ic-agent")]
    #[error("ic agent error: {0}")]
    IcAgentError(#[from] ic_agent::agent::AgentError),

    #[cfg(feature = "pocket-ic")]
    #[error("pocket-ic test error: {0:?}")]
    PocketIcTestError(::pocket_ic::RejectResponse),
}

#[cfg(feature = "pocket-ic")]
impl From<::pocket_ic::RejectResponse> for CanisterClientError {
    fn from(error: ::pocket_ic::RejectResponse) -> Self {
        CanisterClientError::PocketIcTestError(error)
    }
}

pub type CanisterClientResult<T> = Result<T, CanisterClientError>;

/// This tuple is returned incase of IC errors such as Network, canister error.
pub type IcError = ic_cdk::call::Error;

