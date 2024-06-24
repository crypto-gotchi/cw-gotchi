use cosmwasm_std::Coin;
use cw721_base::error::ContractError as Cw721ContractError;
use thiserror::Error;

pub type CResult<T> = std::result::Result<T, ContractError>;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    Cw721(#[from] Cw721ContractError),

    #[error("A minimum expiration day of 1 must be set")]
    MinExpiration {},

    #[error("The magotchi died")]
    MagotchiDied {},

    #[error("The magotchi is not hatched yet")]
    MagotchiUnhatched {},

    #[error("The magotchi is already hatched")]
    MagotchiAlreadyHatched {},

    #[error("Cannot feed with {denom}.")]
    CannotFeedWithDenom { denom: String },

    #[error("Feeding is not free!")]
    FeedingIsNotFree {},

    #[error("Invalid feeing cost {payed:?}, expected {expected:?}")]
    InvalidFeedingCost { payed: Coin, expected: Coin },

    #[error("Not all items are dead")]
    NotAllDead {},
}

impl ContractError {
    pub fn not_found() -> Self {
        ContractError::Std(cosmwasm_std::StdError::not_found("Magotchi not found"))
    }
}
