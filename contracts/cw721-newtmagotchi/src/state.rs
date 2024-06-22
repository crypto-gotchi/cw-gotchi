use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Coin, Timestamp, Uint128};
use cw721::Expiration;
use cw_storage_plus::{Item, Map};

use crate::{
    error::{CResult, ContractError},
    utils::calculate_total_cost,
};

pub const LIVE_STATES: Map<String, LiveState> = Map::new("live_states");
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct LiveState {
    hatched_at: Option<Timestamp>,
    death_time: Timestamp,
}

impl LiveState {
    pub fn new() -> Self {
        Self {
            hatched_at: None,
            death_time: Timestamp::from_nanos(u64::MAX),
        }
    }

    pub fn is_dead(&self, block: &BlockInfo) -> bool {
        Expiration::AtTime(self.death_time).is_expired(block)
    }

    pub fn is_hatched(&self) -> bool {
        self.hatched_at.is_some()
    }

    pub fn hatch(&mut self, block: &BlockInfo) -> CResult<Self> {
        if self.is_hatched() {
            return Err(ContractError::MagotchiAlreadyHatched {});
        }

        self.hatched_at = Some(block.time);
        self.death_time = block.time.plus_days(1);
        Ok(self.to_owned())
    }

    pub fn feed(&mut self, block: &BlockInfo, max_unfed_days: u64) -> CResult<Self> {
        if !self.is_hatched() {
            return Err(ContractError::MagotchiUnhatched {});
        }

        if self.is_dead(block) {
            return Err(ContractError::MagotchiDied {});
        }

        self.death_time = block.time.plus_days(max_unfed_days);
        Ok(self.to_owned())
    }

    pub fn hatched_at(&self) -> Option<Timestamp> {
        self.hatched_at
    }

    pub fn death_time(&self) -> Timestamp {
        self.death_time
    }

    pub fn days_unfed(&self, block: &BlockInfo, max_unfed_days: u64) -> u64 {
        if !self.is_hatched() {
            return 0;
        }

        if self.is_dead(block) {
            return max_unfed_days;
        }

        let days_until_dead = (self.death_time.seconds() - block.time.seconds()) / (24 * 60 * 60);
        max_unfed_days - days_until_dead
    }
}

#[cw_serde]
pub struct Config {
    /// the cost of feeding a magotchi per day
    pub daily_feeding_cost: Vec<Coin>,
    /// the maximum number of days a magotchi can go without food before dying. This is equal to the maximum health
    pub max_unfed_days: u32,
    /// the multiplier of the feeding cost per day in promille
    pub feeding_cost_multiplier: u64,
    /// the cost of hatching a magotchi
    pub graveyard: Addr,
}

impl Config {
    pub fn get_feeding_cost(&self, state: &LiveState, block: &BlockInfo) -> u64 {
        let days_unfed = state.days_unfed(block, self.max_unfed_days as u64);

        let total = calculate_total_cost(days_unfed, self.feeding_cost_multiplier);
        return total;
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
impl Default for Config {
    fn default() -> Self {
        Self {
            daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
            max_unfed_days: 10,
            feeding_cost_multiplier: 1,
            graveyard: Addr::unchecked("graveyard"),
        }
    }
}

#[cfg(test)]
impl LiveState {
    pub fn default() -> Self {
        Self {
            hatched_at: None,
            death_time: Timestamp::from_nanos(u64::MAX),
        }
    }

    pub fn with_hatched_at(days_since_epoch: u64) -> Self {
        Self {
            hatched_at: Some(Timestamp::default().plus_days(days_since_epoch)),
            death_time: Timestamp::default().plus_days(days_since_epoch + 1),
        }
    }

    pub fn custom(hatched_at_days_from_epoch: u64, death_time_days_from_epoch: u64) -> Self {
        Self {
            hatched_at: Some(Timestamp::default().plus_days(hatched_at_days_from_epoch)),
            death_time: Timestamp::default().plus_days(death_time_days_from_epoch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speculoos::prelude::*;

    const ONE_DAY: u64 = 24 * 60 * 60;
    fn mock_block(days_since_epoch: u64) -> BlockInfo {
        BlockInfo {
            time: Timestamp::from_seconds(ONE_DAY * days_since_epoch),
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

            assert_that!(&state.is_dead(&block)).is_false();

            state.death_time = Timestamp::from_seconds(1);
            assert_that!(&state.is_dead(&block)).is_false();

            state.death_time = Timestamp::from_seconds(0);
            assert_that!(&state.is_dead(&block)).is_true();
        }

        #[test]
        fn is_hatched() {
            let state = LiveState::new();
            assert_that!(&state.is_hatched()).is_false();

            let mut state = LiveState::new();
            state.hatched_at = Some(Timestamp::from_seconds(0));
            assert_that!(&state.is_hatched()).is_true();
        }

        #[test]
        fn hatch() {
            let mut state = LiveState::new();
            let block = mock_block(0);

            state = state.hatch(&block).unwrap();
            assert_that!(state.hatched_at())
                .is_some()
                .is_equal_to(Timestamp::default());

            // already hatched
            assert_that!(&state.hatch(&block)).is_err();
        }

        #[test]
        fn feed() {
            let mut state = LiveState::new();
            let block = mock_block(0);

            // unhatched
            assert_that!(&state.feed(&block, 1))
                .is_err()
                .is_equal_to(ContractError::MagotchiUnhatched {});
            state = state.hatch(&block).unwrap();
            // dead
            assert_that!(state.feed(&mock_block(1), 1))
                .is_err()
                .is_equal_to(ContractError::MagotchiDied {});

            // feeding
            assert_that!(state.feed(&mock_block(0), 2))
                .is_ok()
                .is_equal_to(LiveState::custom(0, 2));

            assert_that!(state.feed(&mock_block(1), 2))
                .is_ok()
                .is_equal_to(LiveState::custom(0, 3));

            assert_that!(state.feed(&mock_block(2), 2))
                .is_ok()
                .is_equal_to(LiveState::custom(0, 4));
        }

        #[test]
        fn hatched_at() {
            let state = LiveState::new();
            assert_that!(&state.hatched_at()).is_none();

            let mut state = LiveState::new();
            state.hatched_at = Some(Timestamp::from_seconds(0));
            assert_that!(&state.hatched_at())
                .is_some()
                .is_equal_to(Timestamp::from_seconds(0));
        }

        #[test]
        fn death_time() {
            let state = LiveState::new();
            assert_that!(&state.death_time()).is_equal_to(Timestamp::from_nanos(u64::MAX));

            let mut state = LiveState::new();
            state.death_time = Timestamp::from_seconds(0);
            assert_that!(&state.death_time()).is_equal_to(Timestamp::from_seconds(0));
        }

        #[test]
        fn days_unfed_while_dead() {
            let state = LiveState::new();
            let block = mock_block(0);

            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(0);

            let mut state = LiveState::new();
            state = state.hatch(&block).unwrap();
            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(0);

            // max_unfed_days = 1
            let block = mock_block(1);
            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(1);

            let block = mock_block(200);
            assert_that!(state.is_dead(&block)).is_true();
            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(1);

            // max_unfed_days = 10
            let block = mock_block(11);
            assert_that!(state.is_dead(&block)).is_true();
            assert_that!(&state.days_unfed(&block, 10)).is_equal_to(10);

            let block = mock_block(2000);
            assert_that!(state.is_dead(&block)).is_true();
            assert_that!(&state.days_unfed(&block, 10)).is_equal_to(10);
        }

        #[test]
        fn days_unfed_while_alive() {
            let state = LiveState::new();
            let block = mock_block(0);

            // unhatched
            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(0);

            let state = LiveState::custom(10, 11);
            let block = mock_block(10);
            assert_that!(state.is_dead(&block)).is_false();
            assert_that!(&state.days_unfed(&block, 1)).is_equal_to(0);
            assert_that!(&state.days_unfed(&block, 2)).is_equal_to(1);
            assert_that!(&state.days_unfed(&block, 10)).is_equal_to(9);

            let state = LiveState::custom(10, 12);
            assert_that!(state.is_dead(&block)).is_false();
            assert_that!(&state.days_unfed(&block, 2)).is_equal_to(0);
            assert_that!(&state.days_unfed(&block, 3)).is_equal_to(1);
            assert_that!(&state.days_unfed(&block, 10)).is_equal_to(8);

            let state = LiveState::custom(10, 20);
            assert_that!(state.is_dead(&block)).is_false();
            assert_that!(&state.days_unfed(&block, 10)).is_equal_to(0);
            assert_that!(&state.days_unfed(&block, 11)).is_equal_to(1);
            assert_that!(&state.days_unfed(&block, 20)).is_equal_to(10);
        }
    }

    mod config {
        use super::*;

        #[test]
        fn get_feeding_cost() {
            let config = Config {
                daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
                max_unfed_days: 10,
                feeding_cost_multiplier: 0,
                graveyard: Addr::unchecked("graveyard"),
            };

            let state = LiveState::custom(0, 10);

            assert_that!(&config.get_feeding_cost(&state, &mock_block(0))).is_equal_to(0);
            assert_that!(&config.get_feeding_cost(&state, &mock_block(1))).is_equal_to(1000);
            assert_that!(&config.get_feeding_cost(&state, &mock_block(2))).is_equal_to(2000);

            // dead
            assert_that!(&config.get_feeding_cost(&state, &mock_block(10))).is_equal_to(10_000);

            // with multiplier
            let config = Config {
                daily_feeding_cost: vec![Coin::new(1_000_000, "unewt")],
                max_unfed_days: 10,
                feeding_cost_multiplier: 100,
                graveyard: Addr::unchecked("graveyard"),
            };

            assert_that!(&config.get_feeding_cost(&state, &mock_block(0))).is_equal_to(0);
            assert_that!(config.get_feeding_cost(&state, &mock_block(1))).is_equal_to(1000);
            assert_that!(config.get_feeding_cost(&state, &mock_block(2))).is_equal_to(2100);
        }

        #[test]
        fn get_total_feeding_cost() {
            let config = Config {
                daily_feeding_cost: vec![Coin::new(1_000, "unewt")],
                max_unfed_days: 10,
                feeding_cost_multiplier: 0,
                graveyard: Addr::unchecked("graveyard"),
            };

            let state = LiveState::custom(0, 10);

            assert_that!(&config.get_total_feeding_cost(&state, &mock_block(0), "unewt"))
                .is_ok()
                .is_equal_to(Coin::new(0, "unewt"));

            assert_that!(&config.get_total_feeding_cost(&state, &mock_block(1), "unewt"))
                .is_ok()
                .is_equal_to(Coin::new(1_000_000, "unewt"));

            assert_that!(&config.get_total_feeding_cost(&state, &mock_block(2), "unewt"))
                .is_ok()
                .is_equal_to(Coin::new(2_000_000, "unewt"));

            assert_that!(&config.get_total_feeding_cost(&state, &mock_block(10), "unewt"))
                .is_ok()
                .is_equal_to(Coin::new(10_000_000, "unewt"));
        }
    }
}
