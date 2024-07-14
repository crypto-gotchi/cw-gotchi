use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::marker::PhantomData;
use thiserror::Error;

use cosmwasm_std::{Addr, BlockInfo, CustomMsg, StdError, StdResult, Storage};

use cw721::{ContractInfoResponse, Cw721, Expiration};
use cw_storage_plus::{
    Index, IndexList, IndexedMap, Item, Key, KeyDeserialize, Map, MultiIndex, Prefixer, PrimaryKey,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum WearableOwner {
    Gotchi(String),
    Address(Addr),
}
impl<'a> Prefixer<'a> for WearableOwner {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.key())]
    }
}

#[derive(Error, Debug, PartialEq)]

pub enum WearableOwnerError {
    #[error("Invalid format for WearableOwner: {received}, should be {should_be}")]
    InvalidWearableOwnerFormat { received: String, should_be: String },
    #[error("Invalid WearableOwner type: {ty}")]
    InvalidWearableOwnerType { ty: String },
    #[error("wearable owner is not an address")]
    WearableOwnerIsNotAddress,
}

impl WearableOwnerError {
    pub fn into_std(self) -> StdError {
        StdError::generic_err(self.to_string())
    }
}

fn check_string_length(
    s: &str,
    words: &Vec<&str>,
    classifier: &str,
) -> Result<(), WearableOwnerError> {
    if words.len() != 2 {
        return Err(WearableOwnerError::InvalidWearableOwnerFormat {
            received: s.into(),
            should_be: format!("${classifier}:<value>"),
        });
    }
    Ok(())
}

impl WearableOwner {
    pub fn from_str(s: &str) -> Result<Self, WearableOwnerError> {
        let words: Vec<&str> = s.split(':').collect();
        match words[0] {
            "gotchi" => {
                check_string_length(s, &words, "gotchi")?;
                Ok(WearableOwner::Gotchi(words[1].to_string()))
            }
            "address" => {
                check_string_length(s, &words, "address")?;
                Ok(WearableOwner::Address(Addr::unchecked(words[1])))
            }
            ty => Err(WearableOwnerError::InvalidWearableOwnerType { ty: ty.into() }),
        }
    }

    pub fn address(&self) -> Result<Addr, WearableOwnerError> {
        match self {
            WearableOwner::Address(a) => Ok(a.clone()),
            _ => Err(WearableOwnerError::WearableOwnerIsNotAddress),
        }
    }
}

impl<'a> PrimaryKey<'a> for WearableOwner {
    type Prefix = ();

    type SubPrefix = ();

    type Suffix = Self;

    type SuperSuffix = Self;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = vec![];
        match self {
            WearableOwner::Gotchi(g) => {
                keys.extend("gotchi:".key());
                keys.extend(g.key());
            }
            WearableOwner::Address(a) => {
                keys.extend("address:".key());
                keys.extend(a.key());
            }
        };
        keys
    }
}

impl KeyDeserialize for WearableOwner {
    type Output = WearableOwner;

    fn from_vec(mut value: Vec<u8>) -> StdResult<Self::Output> {
        value.drain(0..2);

        let s = String::from_utf8(value)?;

        WearableOwner::from_str(&s)
            .map_err(|err| StdError::parse_err(type_name::<Self::Output>(), err))
    }
}

pub struct Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub token_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub withdraw_address: Item<'a, String>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

// This is a signal, the implementations are in other files
impl<'a, T, C, E, Q> Cw721<T, C> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
}

impl<T, C, E, Q> Default for Cw721Contract<'static, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
            "withdraw_address",
        )
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(
        contract_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
        withdraw_address_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            withdraw_address: Item::new(withdraw_address_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    /// The owner of the newly minted NFT
    pub owner: WearableOwner,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, WearableOwner, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(_pk: &[u8], d: &TokenInfo<T>) -> WearableOwner {
    d.owner.clone()
}
