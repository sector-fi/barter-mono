use futures::StreamExt;
use reqwest::Client;
use tokio::spawn;
use tokio_tungstenite::connect_async;
use tracing::{error, info};
use url::Url;

pub async fn init_listener(api_key: &str, api_url: &str) {
    let listen_key = get_listen_key(api_key, api_url)
        .await
        .expect("Failed to get listen key");
    let stream_url = format!("wss://fstream.binancefuture.com/ws/{}", listen_key);

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

    spawn(async move {
        if let Err(e) = listen_to_user_data_stream(&stream_url).await {
            error!("Error listening to user data stream: {}", e);
        }
    });
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

async fn listen_to_user_data_stream(stream_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = connect_async(Url::parse(stream_url)?)
        .await
        .map(|(websocket, _)| websocket)
        .expect("Failed to conenct");

    info!("Connected to Binance user data stream");
    println!("stream_url: {}", stream_url);
    let (write, read) = ws_stream.split();

    read.for_each(|message| async {
        if let Ok(msg) = message {
            info!("Received message from Binance: {}", msg.to_text().unwrap());
        }
    })
    .await;
    Ok(())
}
