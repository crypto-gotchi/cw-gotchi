use cosmwasm_std::{BlockInfo, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response};

use crate::{
    error::{CResult, ContractError},
    msg::MagotchiExecuteExtension,
    state::{LiveState, CONFIG, LIVE_STATES},
    Cw721MetadataContract, Metadata,
};

pub fn parse_funds(funds: &Vec<Coin>) -> Result<Coin, ContractError> {
    if funds.is_empty() {
        return Err(ContractError::FeedingIsNotFree {});
    }

    if funds.len() > 1 {
        return Err(ContractError::CannotFeedWithDenom {
            denom: "multiple denominations".to_string(),
        });
    }

    Ok(funds[0].clone())
}

pub fn execute_feed(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
    funds: &Vec<Coin>,
) -> Result<Response, ContractError> {
    let paying_coin = parse_funds(funds)?;

    let config = CONFIG.load(deps.as_ref().storage)?;
    let state = LIVE_STATES.load(deps.storage, token_id.to_string())?;

    let total_feeding_cost = config.get_total_feeding_cost(&state, &env.block, &funds[0].denom)?;

    if paying_coin != total_feeding_cost {
        return Err(ContractError::InvalidFeedingCost {
            payed: paying_coin,
            expected: total_feeding_cost,
        });
    }

    LIVE_STATES.update(deps.storage, token_id.to_string(), |old| match old {
        Some(mut old) => Ok(old.feed(&env.block, config.max_days_without_food.into())?),
        None => Err(ContractError::MagotchiUnhatched {}),
    })?;

    Ok(Response::default().add_attributes(vec![("action", "feed"), ("token_id", token_id)]))
}

pub fn execute_hatch(
    deps: &mut DepsMut,
    env: &Env,
    token_id: &str,
) -> Result<Response, ContractError> {
    LIVE_STATES.update(deps.storage, token_id.to_string(), |old| match old {
        Some(mut old) => Ok(old.hatch(&env.block)?),
        None => Err(ContractError::not_found()),
    })?;

    Ok(Response::default().add_attributes(vec![
        ("action", "hatch"),
        ("token_id", token_id),
        ("birthday", &env.block.time.to_string()),
    ]))
}

pub fn execute_reap(
    deps: &mut DepsMut<Empty>,
    tokens: Option<Vec<String>>,
    env: &Env,
) -> Result<Response, ContractError> {
    let contract = Cw721MetadataContract::default();
    let config = CONFIG.load(deps.as_ref().storage)?;
    let tokens = match tokens {
        Some(tokens) => tokens,
        None => get_all_dead(deps.as_ref(), &env.block),
    };

    tokens.iter().try_for_each(|token_id| -> CResult<()> {
        let state = LIVE_STATES.load(deps.storage, token_id.clone())?;

        // if any are not dead, we cannot reap
        if !state.is_dead(&env.block) {
            return Err(ContractError::NotAllDead {});
        }

        contract
            .tokens
            .update(deps.storage, token_id, |t| match t {
                Some(mut token) => {
                    token.owner = config.graveyard.clone();
                    Ok(token)
                }
                None => Err(ContractError::not_found()),
            })?;
        Ok(())
    })?;
    Ok(Response::default().add_attributes(vec![
        ("action", "reap"),
        ("tokens", tokens.clone().join(",").as_str()),
    ]))
}

pub fn get_all_dead(deps: Deps, block: &BlockInfo) -> Vec<String> {
    LIVE_STATES
        .range(deps.storage, None, None, Order::Ascending)
        .map(Result::unwrap)
        .filter_map(|(token_id, state)| state.is_dead(block).then(|| token_id.to_string()))
        .collect()
}

pub fn execute_mint(
    deps: DepsMut,
    token_id: String,
    env: Env,
    info: MessageInfo,
    msg: cw721_base::ExecuteMsg<Option<Metadata>, MagotchiExecuteExtension>,
) -> Result<Response, ContractError> {
    LIVE_STATES.save(deps.storage, token_id.to_string(), &LiveState::new())?;
    Cw721MetadataContract::default()
        .execute(deps, env, info, msg)
        .map_err(ContractError::from)
}
