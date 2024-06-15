use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomMsg, Uint128};

use crate::state::Config;

#[cw_serde]
pub enum MagotchiExecuteExtension {
    /// Hatch a new magotchi, you need to feed it from now on
    Hatch { token_id: String },
    /// Feed the magotchi, resetting its health
    Feed { token_id: String },
    /// Reap all dead magotchis, sending it to the graveyard. If option tokens is provided, only those tokens will be reaped, otherwise all dead tokens will be reaped (might fail if there are too many dead tokens to reap in one go)
    Reap { tokens: Option<Vec<String>> },
}

impl CustomMsg for MagotchiExecuteExtension {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum MagotchiQueryExtension {
    /// Returns the health of the magotchi
    #[returns(HealthResponse)]
    Health { token_id: String },
    /// Returns the age of the magotchi in days
    #[returns(AgeResponse)]
    Age { token_id: String },
    /// Returns the cost of feeding the magotchi
    #[returns(FeedingCostResponse)]
    FeedingCost { token_id: String },
    /// Returns the Config of the contract, including the daily feeding cost, the maximum days without food and the day length
    #[returns(Config)]
    Config {},
}

impl Default for MagotchiQueryExtension {
    fn default() -> Self {
        MagotchiQueryExtension::Health {
            token_id: '0'.to_string(),
        }
    }
}

impl CustomMsg for MagotchiQueryExtension {}

#[cw_serde]
pub struct HealthResponse {
    pub health: u8,
}

#[cw_serde]
pub struct AgeResponse {
    pub age: u64,
}

#[cw_serde]
pub struct FeedingCostResponse {
    pub cost: Uint128,
}
