use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomMsg, Timestamp, Uint128};
use cw_orch::{ExecuteFns, QueryFns};

use crate::state::{Config, Gotchi, PartialConfig};

#[cw_serde]
#[derive(ExecuteFns)]
pub enum MagotchiExecuteExtension {
    /// Hatch a new magotchi, you need to feed it from now on
    Hatch { token_id: String },
    /// Feed the magotchi, resetting its health
    Feed { token_id: String },
    /// Reap all dead magotchis, sending it to the graveyard. If option tokens is provided, only those tokens will be reaped, otherwise all dead tokens will be reaped (might fail if there are too many dead tokens to reap in one go)
    Reap { tokens: Option<Vec<String>> },
    /// Set the Config of the contract, including the daily feeding cost, the maximum days without food and the day length
    UpdateConfig { config: PartialConfig },
}

impl CustomMsg for MagotchiExecuteExtension {}

#[cw_serde]
#[derive(QueryResponses, QueryFns)]
pub enum MagotchiQueryExtension {
    /// Returns the health of the magotchi
    #[returns(HealthResponse)]
    Health { token_id: String },
    /// Returns the cost of feeding the magotchi
    #[returns(FeedingCostResponse)]
    FeedingCost { token_id: String },
    /// Returns the Config of the contract, including the daily feeding cost, the maximum days without food and the day length
    #[returns(Config)]
    Config {},
    /// Return the birthday of the magotchi
    #[returns(Timestamp)]
    HatchedAt { token_id: String },

    /// Return the dying day of the magotchi
    #[returns(Timestamp)]
    DeathTime { token_id: String },

    /// Return if the magotchi is hatched
    #[returns(bool)]
    IsHatched { token_id: String },

    /// Return the live state of the magotchi
    #[returns(Gotchi)]
    GotchiState { token_id: String },
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
pub struct FeedingCostResponse {
    pub cost: Uint128,
}
