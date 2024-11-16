#[cfg(feature = "ic-agent")]
pub mod agent;

pub mod client;
pub mod error;
pub mod ic_client;

#[cfg(feature = "pocket-ic")]
pub mod pocket_ic;

#[cfg(feature = "ic-agent")]
pub use agent::{AgentError, IcAgentClient};
pub use client::CanisterClient;
pub use error::{CanisterClientError, CanisterClientResult, IcError, IcResult};
#[cfg(feature = "ic-agent")]
pub use ic_agent;
pub use ic_client::IcCanisterClient;
#[cfg(feature = "pocket-ic")]
pub use pocket_ic::PocketIcClient;
