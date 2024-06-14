use cw721::Expiration;
use cw_storage_plus::Map;

pub const LAST_FEDING_EVENTS: Map<String, Expiration> = Map::new("last_feding_events");
pub const BIRTHDAYS: Map<String, Expiration> = Map::new("birthdays");
