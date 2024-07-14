use cosmwasm_std::StdError;
use cw_orch::core::serde_json::error;
use cw_ownable::OwnershipError;
use thiserror::Error;

use crate::state::WearableOwnerError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    Version(#[from] cw2::VersionError),

    #[error(transparent)]
    WearableOwner(#[from] WearableOwnerError),

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("No withdraw address set")]
    NoWithdrawAddress {},
}
