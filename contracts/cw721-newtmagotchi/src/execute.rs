use cosmwasm_std::{BlockInfo, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response};

use crate::{
    error::{CResult, ContractError},
    msg::MagotchiExecuteExtension,
    state::{Gotchi, CONFIG, LIVE_STATES},
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
        Some(mut old) => Ok(old.feed(&env.block, config.max_unfed_days.into())?),
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

pub fn execute_mint(deps: &mut DepsMut, token_id: String) -> Result<(), ContractError> {
    LIVE_STATES
        .save(deps.storage, token_id.to_string(), &Gotchi::new())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::state::Config;

    use super::*;
    use cosmwasm_std::{
        attr, coin, coins,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier},
        Addr, MemoryStorage, OwnedDeps,
    };
    use speculoos::{
        assert_that, boolean::BooleanAssertions, option::OptionAssertions, result::ResultAssertions,
    };

    const TEST_TOKENS: [&str; 3] = ["magotchi1", "magotchi2", "magotchi3"];

    fn prepare() -> OwnedDeps<MemoryStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps.as_mut());
        return deps;
    }

    fn setup_contract(deps: &mut DepsMut) {
        let _ = CONFIG
            .save(
                deps.storage,
                &Config {
                    daily_feeding_cost: vec![Coin::new(1000, "uluna")],
                    max_unfed_days: 10,
                    feeding_cost_multiplier: 0,
                    graveyard: Addr::unchecked("graveyard"),
                },
            )
            .unwrap();

        for token in TEST_TOKENS.iter() {
            let _ = LIVE_STATES
                .save(deps.storage, token.to_string(), &Gotchi::new())
                .unwrap();
        }
    }

    mod hatch {
        use super::*;

        #[test]
        fn test_execute_hatch() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();

            // Execute hatch
            let res = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Verify the response
            assert_that!(res.attributes).is_equal_to(vec![
                attr("action", "hatch"),
                attr("token_id", "magotchi1"),
                attr("birthday", env.block.time.to_string()),
            ]);

            // Verify the state changes
            let state = LIVE_STATES
                .load(&deps.storage, "magotchi1".to_string())
                .unwrap();
            assert_that!(state.is_hatched()).is_true();
            assert_that!(state.is_dead(&env.block)).is_false();
            assert_that!(state.hatched_at())
                .is_some()
                .is_equal_to(env.block.time);

            assert_that!(state.death_time()).is_equal_to(env.block.time.plus_days(1));
        }

        #[test]
        fn test_execute_hatch_already_hatched() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();

            // Hatch the magotchi first
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Try to hatch again
            let res = execute_hatch(&mut deps.as_mut(), &env, "magotchi1");

            // Verify the error
            assert_that!(res).is_err();
        }
    }

    mod feed {
        use super::*;

        #[test]
        fn test_execute_feed() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();

            // Hatch the magotchi first
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Execute feed. After hatch, the magotchi is unfed for 9 days.
            let info = mock_info("feeder", &coins(9_000_000, "uluna"));
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds).unwrap();

            // Verify the response
            assert_that!(res.attributes)
                .is_equal_to(vec![attr("action", "feed"), attr("token_id", "magotchi1")]);

            // Verify the state changes
            let state = LIVE_STATES
                .load(&deps.storage, "magotchi1".to_string())
                .unwrap();
            assert_that!(state.is_hatched()).is_true();
            assert_that!(state.is_dead(&env.block)).is_false();
            assert_that!(state.hatched_at())
                .is_some()
                .is_equal_to(env.block.time);

            // Verify the new death time
            assert_that!(state.death_time()).is_equal_to(env.block.time.plus_days(10));
        }

        #[test]
        fn test_execute_feed_insufficient_funds() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();
            let info = mock_info("feeder", &coins(500, "uluna")); // Insufficient funds

            // Hatch the magotchi first
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Execute feed
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds);

            // Verify the error
            assert_that!(res).is_err();
        }

        #[test]
        fn test_execute_feed_unhatched() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();
            let info = mock_info("feeder", &coins(1000, "uluna"));

            // Execute feed on an unhatched magotchi
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds);

            // Verify the error
            assert_that!(res).is_err();
        }

        #[test]
        fn test_execute_feed_dead() {
            let mut deps = prepare();

            // Mock environment and message info
            let mut env = mock_env();
            let info = mock_info("feeder", &coins(1000, "uluna"));

            // Hatch the magotchi and simulate it being dead
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();
            env.block.time = env.block.time.plus_days(11); // Simulate time passing

            // Execute feed
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds);

            // Verify the error
            assert_that!(res).is_err();
        }

        #[test]
        fn test_execute_feed_wrong_denomination() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();
            let info = mock_info("feeder", &coins(1000, "uusd")); // Wrong denomination

            // Hatch the magotchi first
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Execute feed
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds);

            // Verify the error
            assert_that!(res).is_err();
        }

        #[test]
        fn test_execute_feed_exact_funds() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();
            let info = mock_info("feeder", &coins(9_000_000, "uluna"));

            // Hatch the magotchi first
            let _ = execute_hatch(&mut deps.as_mut(), &env, "magotchi1").unwrap();

            // Execute feed
            let res = execute_feed(&mut deps.as_mut(), &env, "magotchi1", &info.funds).unwrap();

            // Verify the response
            assert_that!(res.attributes)
                .is_equal_to(vec![attr("action", "feed"), attr("token_id", "magotchi1")]);

            // Verify the state changes
            let state = LIVE_STATES
                .load(&deps.storage, "magotchi1".to_string())
                .unwrap();
            assert_that!(state.is_hatched()).is_true();
            assert_that!(state.is_dead(&env.block)).is_false();
            assert_that!(state.hatched_at())
                .is_some()
                .is_equal_to(env.block.time);

            // Verify the new death time
            assert_that!(state.death_time()).is_equal_to(env.block.time.plus_days(10));
        }
    }

    mod reap {
        use cw721_base::{Cw721Contract, InstantiateMsg};

        use crate::{ExecuteMsg, CONTRACT_NAME};
        const SYMBOL: &str = "MAG";
        const MINTER: &str = "minter";

        use super::*;

        fn mint_msg(token_id: &str) -> ExecuteMsg {
            ExecuteMsg::Mint {
                token_id: token_id.to_string(),
                owner: "test_user".to_string(),
                token_uri: None,
                extension: None,
            }
        }

        fn setup_cw721_base() -> OwnedDeps<MemoryStorage, MockApi, MockQuerier> {
            let mut deps = mock_dependencies();
            let contract = Cw721MetadataContract::default();
            let msg = InstantiateMsg {
                name: CONTRACT_NAME.to_string(),
                symbol: SYMBOL.to_string(),
                minter: Some(String::from(MINTER)),
                withdraw_address: None,
            };

            let info = mock_info("creator", &[]);
            let res = contract
                .instantiate(deps.as_mut(), mock_env(), info, msg)
                .unwrap();
            assert_eq!(0, res.messages.len());

            for token in TEST_TOKENS.iter() {
                let msg = mint_msg(token);
                let info = mock_info(MINTER, &[]);
                let _ = contract
                    .execute(deps.as_mut(), mock_env(), info, msg)
                    .unwrap();
            }
            return deps;
        }

        fn prepare_cw721_base_state() -> OwnedDeps<MemoryStorage, MockApi, MockQuerier> {
            let mut deps = setup_cw721_base();
            setup_contract(&mut deps.as_mut());
            return deps;
        }

        #[test]
        fn test_execute_reap() {
            let mut deps = prepare_cw721_base_state();

            // Mock environment and message info
            let mut env = mock_env();

            // Hatch the magotchis and simulate them being dead
            for &token in TEST_TOKENS.iter() {
                let _ = execute_hatch(&mut deps.as_mut(), &env, token).unwrap();
                let state = LIVE_STATES.load(&deps.storage, token.to_string()).unwrap();
                let state = Gotchi::custom(
                    state.hatched_at().unwrap().seconds() / (24 * 60 * 60),
                    env.block.time.plus_seconds(1).seconds() / (24 * 60 * 60),
                );
                LIVE_STATES
                    .save(&mut deps.storage, token.to_string(), &state)
                    .unwrap();
            }
            env.block.time = env.block.time.plus_days(2); // Simulate time passing

            // Execute reap
            let res = execute_reap(&mut deps.as_mut(), None, &env).unwrap();

            // Verify the response
            assert_that!(res.attributes).is_equal_to(vec![
                attr("action", "reap"),
                attr("tokens", "magotchi1,magotchi2,magotchi3"),
            ]);

            // Verify the state changes
            for &token in TEST_TOKENS.iter() {
                let state = LIVE_STATES.load(&deps.storage, token.to_string()).unwrap();
                assert_that!(state.is_dead(&env.block)).is_true();
            }
        }

        #[test]
        fn test_execute_reap_not_all_dead() {
            let mut deps = prepare();

            // Mock environment and message info
            let mut env = mock_env();

            // Hatch the magotchis and simulate only some being dead
            for &token in TEST_TOKENS[..2].iter() {
                let _ = execute_hatch(&mut deps.as_mut(), &env, token).unwrap();
                let state = LIVE_STATES.load(&deps.storage, token.to_string()).unwrap();
                let state = Gotchi::custom(
                    state.hatched_at().unwrap().seconds() / (24 * 60 * 60),
                    env.block.time.plus_seconds(1).seconds() / (24 * 60 * 60),
                );
                LIVE_STATES
                    .save(&mut deps.storage, token.to_string(), &state)
                    .unwrap();
                env.block.time = env.block.time.plus_days(2); // Simulate time passing

                // Execute reap
                let res = execute_reap(&mut deps.as_mut(), None, &env);

                // Verify the error
                assert_that!(res).is_err();
            }
        }

        #[test]
        fn test_execute_reap_empty_tokens() {
            let mut deps = prepare();

            // Mock environment and message info
            let env = mock_env();

            // Execute reap with empty tokens list
            let res = execute_reap(&mut deps.as_mut(), Some(vec![]), &env).unwrap();

            // Verify the response
            assert_that!(res.attributes)
                .is_equal_to(vec![attr("action", "reap"), attr("tokens", "")]);
        }

        #[test]
        fn test_execute_reap_some_dead() {
            let mut deps = prepare_cw721_base_state();

            // Mock environment and message info
            let mut env = mock_env();

            // Hatch the magotchis and simulate only some being dead
            for &token in TEST_TOKENS[..2].iter() {
                let _ = execute_hatch(&mut deps.as_mut(), &env, token).unwrap();
                let state = LIVE_STATES.load(&deps.storage, token.to_string()).unwrap();
                let state = Gotchi::custom(
                    state.hatched_at().unwrap().seconds() / (24 * 60 * 60),
                    env.block.time.plus_seconds(1).seconds() / (24 * 60 * 60),
                );
                LIVE_STATES
                    .save(&mut deps.storage, token.to_string(), &state)
                    .unwrap();
            }
            env.block.time = env.block.time.plus_days(2); // Simulate time passing

            // Execute reap
            let res = execute_reap(
                &mut deps.as_mut(),
                Some(vec!["magotchi1".to_string(), "magotchi2".to_string()]),
                &env,
            )
            .unwrap();

            // Verify the response
            assert_that!(res.attributes).is_equal_to(vec![
                attr("action", "reap"),
                attr("tokens", "magotchi1,magotchi2"),
            ]);
        }
    }

    mod get_all_dead {
        use super::*;
        use speculoos::assert_that;

        #[test]
        fn test_get_all_dead() {
            let mut deps = prepare();

            // Mock environment and message info
            let mut env = mock_env();

            // Hatch the magotchis and simulate them being dead
            for &token in TEST_TOKENS.iter() {
                let _ = execute_hatch(&mut deps.as_mut(), &env, token).unwrap();
                let state = LIVE_STATES.load(&deps.storage, token.to_string()).unwrap();
                let state = Gotchi::custom(
                    state.hatched_at().unwrap().seconds() / (24 * 60 * 60),
                    env.block.time.plus_seconds(1).seconds() / (24 * 60 * 60),
                );
                LIVE_STATES
                    .save(&mut deps.storage, token.to_string(), &state)
                    .unwrap();
            }
            env.block.time = env.block.time.plus_days(2); // Simulate time passing

            // Get all dead tokens
            let dead_tokens = get_all_dead(deps.as_ref(), &env.block);

            // Verify the result
            assert_that!(dead_tokens).is_equal_to(vec![
                "magotchi1".to_string(),
                "magotchi2".to_string(),
                "magotchi3".to_string(),
            ]);
        }
    }

    mod mint {
        use std::borrow::BorrowMut;

        use crate::ExecuteMsg;

        use super::*;
        use cosmwasm_std::{attr, coins, from_binary};
        use speculoos::{assert_that, iter::ContainingIntoIterAssertions};

        #[test]
        fn test_execute_mint() {
            let mut deps = prepare();

            // Mint a new token
            let token_id = "new_magotchi".to_string();

            assert_that!(execute_mint(&mut deps.as_mut(), token_id.clone())).is_ok();

            // Verify the state changes
            let state = LIVE_STATES
                .load(&deps.storage, token_id.to_string())
                .unwrap();
            assert_that!(state.is_hatched()).is_false();
        }
    }

    mod parse_funds {
        use super::*;

        #[test]
        fn test_empty_funds() {
            let funds = vec![];
            let res = parse_funds(&funds);
            assert_that!(res)
                .is_err()
                .is_equal_to(ContractError::FeedingIsNotFree {});
        }

        #[test]
        fn test_multiple_denominations() {
            let funds = vec![coin(1000, "uluna"), coin(1000, "uusd")];
            let res = parse_funds(&funds);
            assert_that!(res)
                .is_err()
                .is_equal_to(ContractError::CannotFeedWithDenom {
                    denom: "multiple denominations".to_string(),
                });
        }

        #[test]
        fn test_single_coin() {
            let funds = vec![coin(1000, "uluna")];
            let res = parse_funds(&funds).unwrap();
            assert_that!(res).is_equal_to(coin(1000, "uluna"));
        }
    }
}
