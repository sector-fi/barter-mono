use reqwest::Error;
use std::time::Duration;
use tokio::time::interval;
use uniswapx_quoter::{quotes::QuoteResponse, UniswapxQuoter};

#[tokio::main]
async fn main() {
    let server = UniswapxQuoter::new();
    let (mut rx, tx) = server.start();

    // Spawn a task to poll the server
    let client = reqwest::Client::new();
    let url: &str = "http://127.0.0.1:8080/quote";

    let num_concurrent_polls = 5; // Number of concurrent polls you want
    let mut count: u32 = 0;
    for _ in 0..num_concurrent_polls {
        let client_clone = client.clone();
        let url_clone = url.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                count += 1;
                let id: u32 = count.clone();
                interval.tick().await; // Wait for the next interval tick
                match poll_server(&client_clone, &url_clone, count).await {
                    Ok(response_body) => {
                        let rx_id = response_body.quote_id.parse::<u32>().ok();
                        match rx_id {
                            Some(rx_id) => {
                                if rx_id == id {
                                    println!("MATCH");
                                } else {
                                    eprintln!("FAILED TO MATCH");
                                }
                            }
                            None => eprintln!("Failed to parse response ID"),
                        }
                    }
                    Err(e) => eprintln!("Error polling server: {}", e),
                }
            }
        });
    }

    loop {
        let result = rx.recv().await;
        match result {
            Some(request) => {
                let response = QuoteResponse {
                    chain_id: request.token_in_chain_id,
                    amount_in: request.amount.clone(),
                    amount_out: request.amount.clone(),
                    filler: "Test filler address".to_string(),
                    request_id: request.request_id,
                    swapper: request.swapper,
                    token_in: request.token_in,
                    token_out: request.token_out,
                    quote_id: request.quote_id,
                };
                tx.send(response).unwrap();
            }
            None => {
                println!("No orders - something has failed");
            }
        }
    }
}

async fn poll_server(client: &reqwest::Client, url: &str, id: u32) -> Result<QuoteResponse, Error> {
    // send a quote request
    let request_payload = r#"
    {
        "requestId": "test-request-id",
        "tokenInChainId": 1,
        "tokenOutChainId": 2,
        "swapper": "test-swapper-address",
        "tokenIn": "test-token-in-address",
        "tokenOut": "test-token-out-address",
        "amount": "100",
        "type_": 1,
        "quoteId": "<ID>"
    }"#
    .replace("<ID>", &id.to_string());
    let res = client
        .post(url)
        .body(request_payload)
        .header("content-type", "application/json")
        .send()
        .await?
        .json::<QuoteResponse>()
        .await?;
    Ok(res)
}
