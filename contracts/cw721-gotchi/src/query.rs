use cosmwasm_std::{Deps, Env, StdResult, Timestamp};

use crate::{
    msg::HealthResponse,
    state::{Gotchi, CONFIG, LIVE_STATES},
};

pub fn query_health(deps: Deps, env: Env, token_id: String) -> StdResult<HealthResponse> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    let config = CONFIG.load(deps.storage)?;

    let days_unfed = state.days_unfed(&env.block, config.max_unfed_days as u64);

    Ok(HealthResponse {
        health: (config.max_unfed_days - days_unfed as u32) as u8,
    })
}

pub fn query_birthday(deps: Deps, token_id: String) -> StdResult<Timestamp> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    match state.hatched_at() {
        Some(birthday) => Ok(birthday),
        None => Err(cosmwasm_std::StdError::not_found("No birthday set")),
    }
}

pub fn query_dying_day(deps: Deps, token_id: String) -> StdResult<Timestamp> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    Ok(state.death_time())
}

pub fn query_is_hatched(deps: Deps, token_id: String) -> StdResult<bool> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    Ok(state.is_hatched())
}

pub fn query_feeding_cost(deps: Deps, env: Env, token_id: String) -> StdResult<u128> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    let config = CONFIG.load(deps.storage)?;
    Ok(config.get_feeding_cost(&state, &env.block).into())
}

pub fn query_live_state(deps: Deps, token_id: String) -> StdResult<Gotchi> {
    LIVE_STATES.load(deps.storage, token_id.clone())
}
pub fn query_config(deps: Deps) -> StdResult<crate::state::Config> {
    CONFIG.load(deps.storage)
}
