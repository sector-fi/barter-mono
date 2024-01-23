use crate::event::{DataKind, MarketEvent};
use barter_integration::model::{
    instrument::{kind::InstrumentKind, Instrument},
    Exchange,
};
use dotenv::dotenv;
use ethers::{
    contract::abigen,
    core::types::Address,
    providers::{Provider, StreamExt, Ws},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info};

abigen!(
    IERC20,
    r#"[
      event Transfer(address indexed from, address indexed to, uint256 value)
      event Approval(address indexed owner, address indexed spender, uint256 value)
  ]"#,
);

const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

pub async fn get_erc20_events() -> Result<UnboundedReceiver<MarketEvent<DataKind>>> {
    dotenv().ok();
    let wss_url = std::env::var("WSS_URL").expect("WSS_URL must be set.");

    let (tx, rx) = mpsc::unbounded_channel();

    info!("connecting to WSS provider at {}", wss_url);
    let provider = Provider::<Ws>::connect(wss_url)
        .await
        .expect("could not connect to WSS provider");
    let client = Arc::new(provider);
    let address: Address = WETH_ADDRESS.parse()?;
    let contract = IERC20::new(address, client);

    info!("subscribing to erc20 events");
    listen_all_events(&contract, tx.clone())
        .await
        .expect("could not get erc20 events");

    Ok(rx)
}

/// Given a contract instance, subscribe to all possible events.
/// This allows to centralize the event handling logic and dispatch
/// proper actions.
///
/// Note that all event bindings have been generated
/// by abigen. Feel free to investigate the abigen expanded code to
/// better understand types and functionalities.
async fn listen_all_events(
    contract: &IERC20<Provider<Ws>>,
    tx: UnboundedSender<MarketEvent<DataKind>>,
) -> Result<()> {
    let events = contract.event::<TransferFilter>().from_block(19065580);
    let mut stream = events.stream().await?.with_meta().take(1);

    while let Some(Ok((evt, meta))) = stream.next().await {
        // println!("{evt:?} {meta:?}");
        tx.send(MarketEvent {
            exchange_time: chrono::Utc::now(),
            received_time: chrono::Utc::now(),
            exchange: Exchange::from("Erc20"),
            instrument: Instrument::new(
                meta.address.to_string(),
                "USDT".to_string(),
                InstrumentKind::Erc20,
            ),
            kind: DataKind::Erc20Transfer(evt),
        })?
    }

    Ok(())
}

// Implement dummy Serializiation (not used when running code)
impl<'de> Deserialize<'de> for TransferFilter {
    fn deserialize<D>(_deserializer: D) -> Result<TransferFilter, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!("Deserialize is not implemented for TransferFilter")
    }
}

impl Serialize for TransferFilter {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!("Serialize is not implemented for TransferFilter")
    }
}
