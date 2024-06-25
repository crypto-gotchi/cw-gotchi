use cw721_base::InstantiateMsg;
use cw_orch::{anyhow, daemon::Daemon, prelude::*, tokio::runtime::Runtime};

const NETWORK: ChainInfo = networks::PION_1;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let daemon = Daemon::builder().chain(NETWORK).build().unwrap();

    let contract = cw721_gotchi::interface::CwGotchi::new("cw721_gotchi", daemon);
    let id = contract.upload()?;

    let init_msg = InstantiateMsg {
        name: "My NFTs".to_string(),
        symbol: "NFT".to_string(),
        minter: Some("neutron1st52glkuvm2dymc5xzuynkfcvy907zfsltm4d0".to_string()),
        withdraw_address: Some("neutron1st52glkuvm2dymc5xzuynkfcvy907zfsltm4d0".to_string()),
    };

    let response = contract.instantiate(&init_msg, None, Some(&[]))?;

    println!("Contract deployed!");

    Ok(())
}
