use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw721::Expiration;
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;

pub const ONE_DAY: Duration = Duration::Time(24 * 60 * 60);

pub const LAST_FEDING_EVENTS: Map<String, Expiration> = Map::new("last_feding_events");
pub const BIRTHDAYS: Map<String, Expiration> = Map::new("birthdays");
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    /// the cost of feeding a magotchi per day
    pub daily_feeding_cost: Vec<Coin>,
    /// the maximum number of days a magotchi can go without food before dying. This is equal to the maximum health
    pub max_days_without_food: u32,
    /// the length of a day in seconds
    pub day_length: Duration,
    /// the multiplier of the feeding cost per day in promille
    pub feeding_cost_multiplier: u64,
    /// the cost of hatching a magotchi
    pub graveyard: Addr,
}
