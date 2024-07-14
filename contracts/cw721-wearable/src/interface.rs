use cw_orch::environment::ChainInfoOwned;
// ANCHOR: custom_interface
use cw_orch::{interface, prelude::*};

use crate::msg::{ExecuteMsg as GenExecuteMsg, InstantiateMsg, QueryMsg as GenQueryMsg};
use crate::Extension;
use cosmwasm_std::Empty;

pub type ExecuteMsg = GenExecuteMsg<Extension, Empty>;
pub type QueryMsg = GenQueryMsg<Empty>;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Cw721;

impl<Chain> Uploadable for Cw721<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cw721_non_transferable")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                crate::entry::execute,
                crate::entry::instantiate,
                crate::entry::query,
            )
            .with_migrate(crate::entry::migrate),
        )
    }
}
