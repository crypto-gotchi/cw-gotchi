use cw721_base::InstantiateMsg;
use cw_orch::interface;

use cw_orch::{anyhow, prelude::*, tokio::runtime::Runtime};

use cw_orch::daemon::{self, Daemon};

fn deploy(daemon: Daemon) -> anyhow::Result<()> {
    let contract = cw721_gotchi::interface::CwGotchi::new("cw721_gotchi", networks::LOCAL_NEUTRON);
    let contract = contract.upload(daemon)?;

    let init_msg = InstantiateMsg {
        name: "My NFTs".to_string(),
        symbol: "NFT".to_string(),
        minter: Some("neutron1st52glkuvm2dymc5xzuynkfcvy907zfsltm4d0".to_string()),
        withdraw_address: Some("neutron1st52glkuvm2dymc5xzuynkfcvy907zfsltm4d0".to_string()),
    };

    let response = contract.instantiate(init_msg, None, &[], &[])?;

    log::info!("Contract deployed at {}", response.contract_address);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let runtime = Runtime::new()?;

    let daemon = Daemon::builder()
        .chain(networks::LOCAL_NEUTRON)
        .handle(runtime.handle())
        .build()
        .unwrap();

    deploy(daemon)?;

    Ok(())
}
