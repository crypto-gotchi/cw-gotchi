use anyhow::Error;
use cw721_base::InstantiateMsg;
use cw_orch::{anyhow, daemon::Daemon, prelude::*, tokio::runtime::Runtime};

const NETWORK: ChainInfo = networks::PION_1;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let daemon = Daemon::builder().chain(NETWORK).build().unwrap();
    let address = daemon.wallet().address()?;

    let contract = cw721_gotchi::interface::CwGotchi::new("cw721_gotchi", daemon);
    contract.upload_if_needed()?;

    let init_msg = InstantiateMsg {
        name: "My Test Gotchis".to_string(),
        symbol: "GOTCHI".to_string(),
        minter: Some(address.to_string()),
        withdraw_address: Some(address.to_string()),
    };

    let response = contract.instantiate(&init_msg, None, Some(&[]))?;

    println!("Contract deployed!");

    Ok(())
}
