use cw721::{AllNftInfoResponse, TokensResponse};
use cw721_base::msg::ExecuteMsgFns;
use cw721_base::InstantiateMsg;
use cw721_gotchi::{msg::MagotchiExecuteExtensionFns, QueryMsg};
use cw721_gotchi::{ExecuteMsg, Metadata};
use cw_orch::{anyhow, core::serde_json, daemon::Daemon, prelude::*, tokio::runtime::Runtime};

const NETWORK: ChainInfo = networks::PION_1;

#[derive(Debug, Clone, serde::Deserialize)]
struct GotchiJson {
    token_id: String,
    token_uri: String,
    owner: String,
    minted: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GotchiJsons {
    first_gotchis: Vec<GotchiJson>,
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let daemon = Daemon::builder().chain(NETWORK).build()?;
    let address = daemon.wallet().address()?;

    let contract = cw721_gotchi::interface::CwGotchi::new("cw721_gotchi", daemon);
    assert!(contract.latest_is_uploaded()?);
    assert!(contract.is_running_latest()?);

    // read the json file first_gotchi.json with "first_gotchis": [ {token_id, token_uri, owner}]
    let json_file = std::fs::read_to_string("first_gotchis.json")?;
    let first_gotchis: GotchiJsons = serde_json::from_str(&json_file)?;

    let index = 5;
    let selected_gotchi = &first_gotchis.first_gotchis[index];

    assert!(selected_gotchi.minted == false);

    let mint_msg: ExecuteMsg = ExecuteMsg::Mint {
        token_id: selected_gotchi.token_id.clone(),
        owner: selected_gotchi.owner.clone(),
        token_uri: Some(selected_gotchi.token_uri.clone()),
        extension: None,
    };

    let res = contract.execute(&mint_msg, None)?;

    Ok(())
}
