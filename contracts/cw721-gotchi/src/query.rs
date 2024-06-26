use cosmwasm_std::{Deps, Env, StdResult, Timestamp};

use crate::{
    msg::HealthResponse,
    state::{Gotchi, CONFIG, LIVE_STATES},
};

pub fn query_health(deps: Deps, env: Env, token_id: String) -> StdResult<HealthResponse> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    let config = CONFIG.load(deps.storage)?;

    let health = state.health(&env.block, config.max_unfed_days as u64) as u8;
    Ok(HealthResponse { health })
}

pub fn query_hatched_at(deps: Deps, token_id: String) -> StdResult<Timestamp> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    match state.hatched_at() {
        Some(hatched_at) => Ok(hatched_at),
        None => Err(cosmwasm_std::StdError::not_found("No birthday set")),
    }
}

pub fn query_death_time(deps: Deps, token_id: String) -> StdResult<Timestamp> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    Ok(state.death_time())
}

pub fn query_is_hatched(deps: Deps, token_id: String) -> StdResult<bool> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    Ok(state.is_hatched())
}

pub fn query_is_alive(deps: Deps, token_id: String, env: Env) -> StdResult<bool> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    Ok(!state.is_dead(&env.block))
}

pub fn query_feeding_cost(deps: Deps, env: Env, token_id: String) -> StdResult<u128> {
    let state = LIVE_STATES.load(deps.storage, token_id.clone())?;
    let config = CONFIG.load(deps.storage)?;
    Ok(config.get_feeding_cost(&state, &env.block).into())
}

pub fn query_gotchi_state(deps: Deps, token_id: String) -> StdResult<Gotchi> {
    LIVE_STATES.load(deps.storage, token_id.clone())
}
pub fn query_config(deps: Deps) -> StdResult<crate::state::Config> {
    CONFIG.load(deps.storage)
}
