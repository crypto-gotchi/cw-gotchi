use cw721_base::msg::ExecuteMsgFns;
use cw721_base::InstantiateMsg;
use cw721_gotchi::msg::MagotchiExecuteExtensionFns;
use cw721_gotchi::ExecuteMsg;
use cw_orch::{anyhow, daemon::Daemon, prelude::*, tokio::runtime::Runtime};

const NETWORK: ChainInfo = networks::PION_1;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let daemon = Daemon::builder().chain(NETWORK).build()?;
    let address = daemon.wallet().address()?;

    let contract = cw721_gotchi::interface::CwGotchi::new("cw721_gotchi", daemon);
    assert!(contract.latest_is_uploaded()?);
    assert!(contract.is_running_latest()?);

    let mint_msg: ExecuteMsg = ExecuteMsg::Extension {
        msg: cw721_gotchi::msg::MagotchiExecuteExtension::UpdateConfig {
            config: {
                cw721_gotchi::state::PartialConfig {
                    daily_feeding_cost: Some(vec![Coin::new(1, "untrn".to_string())]),
                    max_unfed_days: None,
                    feeding_cost_multiplier: Some(0),
                    graveyard: None,
                }
            },
        },
    };

    let res = contract.execute(&mint_msg, None)?;

    Ok(())
}
