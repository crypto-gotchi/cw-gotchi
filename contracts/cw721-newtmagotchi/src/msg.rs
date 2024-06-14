use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CustomMsg, Uint128};

#[cw_serde]
pub enum MagotchiExecuteExtension {
    Hatch { token_id: String },
    Feed { token_id: String },
    Reap {},
}

impl CustomMsg for MagotchiExecuteExtension {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum MagotchiQueryExtension {
    #[returns(HealthResponse)]
    Health { token_id: String },
    #[returns(AgeResponse)]
    Age { token_id: String },
    #[returns(FeedingCostResponse)]
    FeedingCost { token_id: String },
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
