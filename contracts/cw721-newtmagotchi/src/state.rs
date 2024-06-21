use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin, Timestamp, Uint128};
use cw721::Expiration;
use cw_storage_plus::{Item, Map};

use crate::error::{CResult, ContractError};

pub const LIVE_STATES: Map<String, LiveState> = Map::new("live_states");
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct LiveState {
    birthday: Option<Timestamp>,
    dying_day: Timestamp,
}

impl LiveState {
    pub fn new() -> Self {
        Self {
            birthday: None,
            dying_day: Timestamp::from_nanos(u64::MAX),
        }
    }

    pub fn is_dead(&self, block: &BlockInfo) -> bool {
        Expiration::AtTime(self.dying_day).is_expired(block)
    }

    pub fn is_hatched(&self) -> bool {
        self.birthday.is_some()
    }

    pub fn hatch(&mut self, block: &BlockInfo) -> CResult<Self> {
        if self.is_hatched() {
            return Err(ContractError::MagotchiAlreadyHatched {});
        }

        self.birthday = Some(block.time);
        self.dying_day = block.time.plus_days(1);
        Ok(self.to_owned())
    }

    pub fn feed(&mut self, block: &BlockInfo, max_days_without_food: u64) -> CResult<Self> {
        if !self.is_hatched() {
            return Err(ContractError::MagotchiUnhatched {});
        }

        if self.is_dead(block) {
            return Err(ContractError::MagotchiDied {});
        }

        self.dying_day = block.time.plus_seconds(max_days_without_food);
        Ok(self.to_owned())
    }

    pub fn birthday(&self) -> Option<Timestamp> {
        self.birthday
    }

    pub fn dying_day(&self) -> Timestamp {
        self.dying_day
    }

    pub fn days_unfed(&self, block: &BlockInfo, max_days_without_food: u64) -> u64 {
        if !self.is_hatched() {
            return 0;
        }

        if self.is_dead(block) {
            return max_days_without_food;
        }

        let seconds_unfed =
            self.dying_day.minus_days(max_days_without_food).seconds() - block.time.seconds();

        seconds_unfed / (24 * 60 * 60)
    }
}

#[cw_serde]
pub struct Config {
    /// the cost of feeding a magotchi per day
    pub daily_feeding_cost: Vec<Coin>,
    /// the maximum number of days a magotchi can go without food before dying. This is equal to the maximum health
    pub max_days_without_food: u32,
    /// the multiplier of the feeding cost per day in promille
    pub feeding_cost_multiplier: u64,
    /// the cost of hatching a magotchi
    pub graveyard: Addr,
}

impl Config {
    pub fn get_feeding_cost(&self, state: &LiveState, block: &BlockInfo) -> u64 {
        let days_unfed = state.days_unfed(block, self.max_days_without_food as u64);
        return self.feeding_cost_multiplier.pow(days_unfed as u32);
    }

    pub fn get_total_feeding_cost(
        &self,
        state: &LiveState,
        block: &BlockInfo,
        denom: &str,
    ) -> CResult<Coin> {
        let cost = self.get_feeding_cost(state, block);

        let feeding_price = self
            .daily_feeding_cost
            .iter()
            .find(|coin| coin.denom == denom)
            .ok_or(ContractError::CannotFeedWithDenom {
                denom: denom.to_string(),
            })?
            .amount
            .saturating_mul(Uint128::from(cost));

        Ok(Coin {
            denom: denom.to_string(),
            amount: feeding_price,
        })
    }
}

#[cfg(test)]
impl Config {
    pub fn new() -> Self {
        Self {
            daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
            max_days_without_food: 10,
            feeding_cost_multiplier: 1,
            graveyard: Addr::unchecked("graveyard"),
        }
    }
}

#[cfg(test)]
impl LiveState {
    pub fn default() -> Self {
        Self {
            birthday: None,
            dying_day: Timestamp::from_nanos(u64::MAX),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speculoos::prelude::*;

    fn mock_block(s_since_epoch: u64) -> BlockInfo {
        BlockInfo {
            time: Timestamp::from_seconds(s_since_epoch),
            height: 0,
            chain_id: "neutron-1".to_owned(),
        }
    }

    mod live_state {
        use super::*;

        #[test]
        fn is_dead() {
            let mut state = LiveState::new();
            let block = mock_block(0);

            assert_that(&state.is_dead(&block)).is_false();

            state.dying_day = Timestamp::from_seconds(1);
            assert_that(&state.is_dead(&block)).is_false();

            state.dying_day = Timestamp::from_seconds(0);
            assert_that(&state.is_dead(&block)).is_true();
        }

        #[test]
        fn is_hatched() {
            let state = LiveState::new();
            assert_that(&state.is_hatched()).is_false();

            let mut state = LiveState::new();
            state.birthday = Some(Timestamp::from_seconds(0));
            assert_that(&state.is_hatched()).is_true();
        }

        #[test]
        fn hatch() {
            let mut state = LiveState::new();
            let block = mock_block(0);

            assert_that(&state.hatch(&block)).is_err();
            assert_that(&state.hatch(&block)).is_err();
        }

        #[test]
        fn feed() {
            let mut state = LiveState::new();
            let block = mock_block(0);

            assert_that(&state.feed(&block, 1)).is_err();

            state.hatch(&block).unwrap();
            assert_that(&state.feed(&block, 1)).is_ok();
            assert_that(&state.feed(&block, 1)).is_err();
        }

        #[test]
        fn birthday() {
            let state = LiveState::new();
            assert_that(&state.birthday()).is_none();

            let mut state = LiveState::new();
            state.birthday = Some(Timestamp::from_seconds(0));
            assert_that(&state.birthday())
                .is_some()
                .is_equal_to(Timestamp::from_seconds(0));
        }

        #[test]
        fn dying_day() {
            let state = LiveState::new();
            assert_that(&state.dying_day()).is_equal_to(Timestamp::from_nanos(u64::MAX));

            let mut state = LiveState::new();
            state.dying_day = Timestamp::from_seconds(0);
            assert_that(&state.dying_day()).is_equal_to(Timestamp::from_seconds(0));
        }
    }

    mod config {
        use super::*;

        #[test]
        fn get_feeding_cost() {
            let config = Config {
                daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
                max_days_without_food: 10,
                feeding_cost_multiplier: 2,
                graveyard: Addr::unchecked("graveyard"),
            };

            let state = LiveState::new();
            let block = mock_block(0);

            assert_that(&config.get_feeding_cost(&state, &block)).is_equal_to(1);

            let block = mock_block(1);
            assert_that(&config.get_feeding_cost(&state, &block)).is_equal_to(2);
        }

        #[test]
        fn get_total_feeding_cost() {
            let config = Config {
                daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
                max_days_without_food: 10,
                feeding_cost_multiplier: 2,
                graveyard: Addr::unchecked("graveyard"),
            };

            let state = LiveState::new();
            let block = mock_block(0);

            assert_that(
                &config
                    .get_total_feeding_cost(&state, &block, "unewt")
                    .unwrap(),
            )
            .is_equal_to(Coin::new(1_000_000, "unewt"));

            assert_that(
                &config
                    .get_total_feeding_cost(&state, &block, "utest")
                    .is_err(),
            )
            .is_true();
        }
    }
}
