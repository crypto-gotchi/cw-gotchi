use cosmwasm_std::{Coin, Deps, DepsMut, Empty, Env, Response};
use cw721::Expiration;
use cw721_base::error::ContractError as Cw721ContractError;
use cw_utils::Duration;

use crate::{
    error::ContractError,
    state::{BIRTHDAYS, CONFIG, LAST_FEDING_EVENTS, ONE_DAY},
};

pub fn register_birthday(
    deps: &mut DepsMut<Empty>,
    env: &Env,
    token_id: &str,
) -> Result<(), ContractError> {
    BIRTHDAYS.update(deps.storage, token_id.to_string(), |old| match old {
        Some(_) => Err(Cw721ContractError::Claimed {}),
        None => Ok(Expiration::AtHeight(env.block.height)),
    })?;

    Ok(())
}

pub fn feed(deps: &mut DepsMut, token_id: &str, funds: &Vec<Coin>) -> Result<(), ContractError> {
    // implement feeding logic here

    Ok(())
}

pub fn check_if_alive(deps: &Deps, env: &Env, token_id: &str) -> Result<(), ContractError> {
    // implement checking if the magotchi is alive here
    let last_fed_at = LAST_FEDING_EVENTS
        .may_load(deps.storage, token_id.to_string())?
        .ok_or(ContractError::MagotchiUnhatched {})?;

    let config = CONFIG.load(deps.storage)?;

    let dying_day = last_fed_at + ONE_DAY * config.max_days_without_food as u64;
    if dying_day?.is_expired(&env.block) {
        return Err(ContractError::MagotchiDied {});
    }

    Ok(())
}

pub fn execute_feed(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
    funds: &Vec<Coin>,
) -> Result<Response, ContractError> {
    check_if_alive(&deps.as_ref(), &env, &token_id)?;
    feed(deps, &token_id, &funds)?;

    Ok(Response::default().add_attributes(vec![("action", "feed"), ("token_id", token_id)]))
}

pub fn set_first_last_fed_event(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
) -> Result<Expiration, ContractError> {
    let last_fed_at = Expiration::AtTime(env.block.time.minus_days(1));
    LAST_FEDING_EVENTS.save(deps.storage, token_id.to_string(), &last_fed_at)?;

    Ok(last_fed_at)
}

pub fn check_hachable(deps: &Deps, token_id: &str) -> Result<(), ContractError> {
    match LAST_FEDING_EVENTS.may_load(deps.storage, token_id.to_string())? {
        Some(_) => Err(ContractError::MagotchiAlreadyHatched {}),
        None => Ok(()),
    }
}

pub fn execute_hatch(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
) -> Result<Response, ContractError> {
    check_hachable(&deps.as_ref(), &token_id)?;
    let last_fed_event = set_first_last_fed_event(deps, &env, &token_id)?;

    Ok(Response::default().add_attributes(vec![
        ("action", "hatch"),
        ("token_id", token_id),
        ("last_fed_at", &last_fed_event.to_string()),
    ]))
}
