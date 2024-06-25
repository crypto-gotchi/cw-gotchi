use cw721_base::InstantiateMsg;
use cw_orch::environment::ChainInfoOwned;
use cw_orch::{interface, prelude::*};

use crate::{ExecuteMsg, QueryMsg};
use cosmwasm_std::Empty;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct CwGotchi;

impl<Chain> Uploadable for CwGotchi<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cw721_gotchi.wasm")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            crate::entry::execute,
            crate::entry::instantiate,
            crate::entry::query,
        ))
    }
}
