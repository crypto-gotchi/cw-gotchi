use cosmwasm_schema::cw_serde;
use cosmwasm_std::Empty;
pub use cw721_base::{InstantiateMsg, MinterResponse};
use msg::{MagotchiExecuteExtension, MagotchiQueryExtension};
pub mod error;
pub mod execute;
pub mod msg;
pub mod query;
pub mod state;
pub mod utils;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw721-newtmagotchi";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

pub type Extension = Option<Metadata>;

pub type Cw721MetadataContract<'a> = cw721_base::Cw721Contract<
    'a,
    Extension,
    Empty,
    MagotchiExecuteExtension,
    MagotchiQueryExtension,
>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, MagotchiExecuteExtension>;
pub type QueryMsg = cw721_base::QueryMsg<MagotchiQueryExtension>;

pub mod entry {

    use std::borrow::BorrowMut;

    use super::*;

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{
        to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    };
    use error::ContractError;
    use execute::{execute_feed, execute_hatch, execute_mint, execute_reap};
    use state::{Gotchi, LIVE_STATES};

    // This makes a conscious choice on the various generics used by the contract
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        Cw721MetadataContract::default()
            .instantiate(deps.branch(), env, info, msg)
            .map_err(ContractError::from)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg.clone() {
            ExecuteMsg::Extension { msg } => match msg {
                MagotchiExecuteExtension::Feed { token_id } => {
                    execute_feed(&mut deps, &env, &token_id, &info.funds)
                }
                MagotchiExecuteExtension::Hatch { token_id } => {
                    execute_hatch(&mut deps, &env, &token_id)
                }
                MagotchiExecuteExtension::Reap { tokens } => execute_reap(&mut deps, tokens, &env),
            },
            ExecuteMsg::Mint {
                token_id,
                owner: _,
                token_uri: _,
                extension: _,
            } => {
                // Initialize the live state for the token. No need to check if it already exists, cause the cw721 base contract will fail if it does.
                execute_mint(deps.borrow_mut(), token_id)?;
                Cw721MetadataContract::default()
                    .execute(deps, env, info, msg)
                    .map_err(ContractError::from)
            }

            _ => Cw721MetadataContract::default()
                .execute(deps, env, info, msg)
                .map_err(ContractError::from),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                MagotchiQueryExtension::Health { token_id } => {
                    to_json_binary(&query::query_health(deps, env, token_id)?)
                }
                MagotchiQueryExtension::FeedingCost { token_id } => {
                    to_json_binary(&query::query_feeding_cost(deps, env, token_id)?)
                }
                MagotchiQueryExtension::Config {} => to_json_binary(&query::query_config(deps)?),
                MagotchiQueryExtension::Birthday { token_id } => {
                    to_json_binary(&query::query_birthday(deps, token_id)?)
                }
                MagotchiQueryExtension::DyingDay { token_id } => {
                    to_json_binary(&query::query_dying_day(deps, token_id)?)
                }
                MagotchiQueryExtension::IsHatched { token_id } => {
                    to_json_binary(&query::query_is_hatched(deps, token_id)?)
                }
                MagotchiQueryExtension::LiveState { token_id } => {
                    to_json_binary(&query::query_live_state(deps, token_id)?)
                }
            },
            _ => Cw721MetadataContract::default().query(deps, env, msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::Cw721Query;

    const CREATOR: &str = "creator";

    /// Make sure cw2 version info is properly initialized during instantiation,
    /// and NOT overwritten by the base contract.
    #[test]
    fn proper_cw2_initialization() {
        let mut deps = mock_dependencies();

        entry::instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("larry", &[]),
            InstantiateMsg {
                name: "".into(),
                symbol: "".into(),
                minter: None,
                withdraw_address: None,
            },
        )
        .unwrap();

        let version = cw2::get_contract_version(deps.as_ref().storage).unwrap();
        assert_eq!(version.contract, CONTRACT_NAME);
        assert_ne!(version.contract, cw721_base::CONTRACT_NAME);
    }

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id = "Enterprise";
        let token_uri = Some("https://starships.example.com/Starship/Enterprise.json".into());
        let extension = Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        });
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: token_uri.clone(),
            extension: extension.clone(),
        };
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, token_uri);
        assert_eq!(res.extension, extension);
    }
}
