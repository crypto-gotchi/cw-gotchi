use cosmwasm_std::{Coin, Deps, DepsMut, Empty, Env, Order, Response, StdResult};
use cw721::Expiration;
use cw721_base::error::ContractError as Cw721ContractError;
use cw_utils::Duration;

use crate::{
    error::ContractError,
    state::{BIRTHDAYS, CONFIG, LAST_FEDING_EVENTS, ONE_DAY},
    Cw721MetadataContract, Metadata,
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

pub fn check_if_alive(
    deps: &Deps,
    env: &Env,
    token_id: &str,
    max_days_without_food: u64,
) -> Result<(), ContractError> {
    // implement checking if the magotchi is alive here
    let dying_day = get_dying_day(deps, token_id, max_days_without_food)?;
    if dying_day.is_expired(&env.block) {
        return Err(ContractError::MagotchiDied {});
    }

    Ok(())
}

pub fn get_dying_day(
    deps: &Deps<Empty>,
    token_id: &str,
    max_days_without_food: u64,
) -> Result<Expiration, ContractError> {
    let last_fed_at = LAST_FEDING_EVENTS
        .may_load(deps.storage, token_id.to_string())?
        .ok_or(ContractError::MagotchiUnhatched {})?;

    let dying_day = calculate_dying_day(last_fed_at, max_days_without_food as u64)?;
    Ok(dying_day)
}

fn calculate_dying_day(
    last_fed_at: Expiration,
    max_days_without_food: u64,
) -> StdResult<Expiration> {
    last_fed_at + ONE_DAY * max_days_without_food
}

pub fn execute_feed(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
    funds: &Vec<Coin>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.as_ref().storage)?;
    check_if_alive(
        &deps.as_ref(),
        &env,
        &token_id,
        config.max_days_without_food.into(),
    )?;
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

pub fn execute_reap(
    deps: &mut DepsMut<Empty>,
    tokens: Option<Vec<String>>,
    env: &Env,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.as_ref().storage)?;
    let tokens = match tokens {
        Some(tokens) => tokens,
        None => Cw721MetadataContract::default()
            .tokens
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<String>>>()?,
    };
    let mut harverst = Vec::new();
    for token_id in tokens {
        let contract = Cw721MetadataContract::default();
        let dying_day = get_dying_day(
            &deps.as_ref(),
            &token_id,
            config.max_days_without_food.into(),
        )?;
        if dying_day.is_expired(&env.block) {
            // transfer the token to the graveyard
            let mut token: cw721_base::state::TokenInfo<Option<Metadata>> =
                contract.tokens.load(deps.storage, &token_id)?;
            token.owner = config.graveyard.clone();
            contract.tokens.save(deps.storage, &token_id, &token)?;
            harverst.push(token_id);
        }
    }
    Ok(Response::default().add_attributes(vec![
        ("action", "reap"),
        ("tokens", harverst.join(",").as_str()),
    ]))
}
