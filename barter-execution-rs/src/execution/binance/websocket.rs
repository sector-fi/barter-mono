use crate::{
    execution::binance::types::{order_update::BinanceFutOrderStatus, BinanceFuturesEventType},
    model::AccountEventKind,
};

use super::types::{
    account_update::BinanceAccountUpdate, order_update::BinanceFutOrderUpdate,
    BinanceFutAccountEvent,
};
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};
use url::Url;

pub async fn init_listener(api_key: &str, api_url: &str) -> BoxStream<'static, AccountEventKind> {
    let listen_key = get_listen_key(api_key, api_url)
        .await
        .expect("Failed to get listen key");
    let stream_url = format!("wss://fstream.binancefuture.com/ws/{}", listen_key);
    let stream_url = Url::parse(&stream_url).expect("Failed to parse stream url");

    // TODO:
    // PUT /fapi/v1/listenKey
    // Keepalive a user data stream to prevent a time out.
    // User data streams will close after 60 minutes.
    // It's recommended to send a ping about every 60 minutes.
    // If response -1125 error "This listenKey does not exist."
    // Please use POST /fapi/v1/listenKey to recreate listenKey and use new listenKey to build connection.

    // let mut interval = interval(Duration::from_secs(60 * 30)); // Renew every 30 minutes
    // loop {
    //     interval.tick().await;

    // spawn(async move {
    //     if let Err(e) = listen_to_user_data_stream(&stream_url).await {
    //         error!("Error listening to user data stream: {}", e);
    //     }
    // });
    listen_to_user_data_stream(stream_url).await
}

async fn get_listen_key(
    api_key: &str,
    api_url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .post(format!("{api_url}/fapi/v1/listenKey"))
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    info!("Got Listener Key from Binance: {:?}", res);
    Ok(res["listenKey"].as_str().unwrap().to_string())
}

fn process_message(msg: Message) -> Vec<AccountEventKind> {
    match msg {
        Message::Text(text) => {
            let event: BinanceFutAccountEvent = serde_json::from_str(&text).unwrap();
            let mut events = Vec::new();

            match event.event_type {
                BinanceFuturesEventType::AccountUpdate => {
                    match serde_json::from_str::<BinanceAccountUpdate>(&text) {
                        Ok(account_update) => {
                            let (bal_event, pos_event) =
                                <(AccountEventKind, AccountEventKind)>::from(account_update);
                            events.push(bal_event);
                            events.push(pos_event);
                        }
                        Err(e) => {
                            error!("Failed to parse account update: {}", e);
                        }
                    }
                }
                BinanceFuturesEventType::OrderTradeUpdate => {
                    match serde_json::from_str::<BinanceFutOrderUpdate>(&text) {
                        Ok(trade_update) => match trade_update.order.order_status {
                            BinanceFutOrderStatus::Filled => {
                                events.push(trade_update.into());
                            }
                            BinanceFutOrderStatus::PartiallyFilled => {
                                events.push(trade_update.into());
                            }
                            _ => {}
                        },
                        Err(e) => {
                            error!("Failed to parse order trade update: {}", e);
                        }
                    }
                }
            }
            events
        }
        _ => Vec::new(), // Non-text messages generate no events
    }
}

async fn listen_to_user_data_stream(ws_url: Url) -> BoxStream<'static, AccountEventKind> {
    let ws_stream = connect_async(ws_url)
        .await
        .map(|(websocket, _)| websocket)
        .expect("Failed to conenct to Binance websocket");

    info!("Connected to Binance user data stream");
    ws_stream
        .flat_map(|message| {
            let events = process_message(message.unwrap()); // Safely handle unwrap in real code
            futures::stream::iter(events)
        })
        .boxed()
}
