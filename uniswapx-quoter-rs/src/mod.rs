use crate::quotes::{QuoteRequest, QuoteResponse};
use std::time::Duration;
use tokio::sync::broadcast::{self, Sender};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver},
    time,
};
use warp::{Filter, Reply};

pub mod quotes;

#[derive(Debug)]
pub struct UniswapxQuoter {}

type QuoterTx = Sender<QuoteResponse>;
type QuoterRx = UnboundedReceiver<QuoteRequest>;

impl UniswapxQuoter {
    pub fn new() -> UniswapxQuoter {
        UniswapxQuoter {}
    }

    pub fn start(&self) -> (QuoterRx, QuoterTx) {
        let (server_tx, quoter_rx) = mpsc::unbounded_channel::<QuoteRequest>();
        let (quoter_tx, mut server_rx) = broadcast::channel::<QuoteResponse>(16);
        let local_quoter_tx = quoter_tx.clone();

        // Define the warp filter for the quote endpoint
        let quote_endpoint = warp::path("quote")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(move |request: QuoteRequest| {
                let tx_clone = server_tx.clone();
                let mut rx: broadcast::Receiver<QuoteResponse> = local_quoter_tx.subscribe();
                async move {
                    // 1. Send the request to the subscriber
                    // 2. Set a timeout. If the subscriber doesn't respond in 200ms, return 200 to the server
                    let id = request.quote_id.clone();
                    let _ = tx_clone.send(request.clone());

                    // Await a message with a timeout
                    let timeout = Duration::from_millis(200);
                    loop {
                        tokio::select! {
                            response = rx.recv() => {
                                match response {
                                    Ok(response) => {
                                        if response.quote_id == id {
                                            return Ok::<_, warp::Rejection>(warp::reply::json(&response).into_response())
                                        } else {
                                            continue
                                        }
                                    },
                                    Err(_) => {
                                        return Ok::<_, warp::Rejection>(warp::reply::with_status(warp::reply::json(&"ok"), warp::http::StatusCode::OK).into_response())
                                    },
                                }
                            }
                            _ = time::sleep(timeout) => {
                                return Ok::<_, warp::Rejection>(warp::reply::with_status(warp::reply::json(&"ok"), warp::http::StatusCode::OK).into_response())
                            }
                        }
                    }
                }
            });

        tokio::spawn(async move {
            loop {
                let result = server_rx.recv().await;
                match result {
                    Ok(_) => {}
                    Err(_) => {
                        println!("No orders - something has failed");
                    }
                }
            }
        });

        tokio::spawn(async move {
            println!("Starting warp server");
            // Start the warp server on a specified address and port
            warp::serve(quote_endpoint)
                .run(([127, 0, 0, 1], 8080))
                .await;
        });

        (quoter_rx, quoter_tx)
    }

    pub fn respond(&self, request: QuoteRequest) -> QuoteResponse {
        QuoteResponse {
            chain_id: request.token_in_chain_id,
            amount_in: request.amount.clone(),
            amount_out: request.amount.clone(),
            filler: "HERE's WHERE THE FILLER ADDRESS GOES".to_string(),
            request_id: request.request_id,
            swapper: request.swapper,
            token_in: request.token_in,
            token_out: request.token_out,
            quote_id: request.quote_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{quotes::QuoteResponse, UniswapxQuoter};
    use reqwest::Error;
    use std::time::Duration;
    use tokio::time::interval;

    #[tokio::test]
    async fn test_concurrent_requests() {
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
                    // println!("Main - New Order: {:?}", request);

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

    async fn poll_server(
        client: &reqwest::Client,
        url: &str,
        id: u32,
    ) -> Result<QuoteResponse, Error> {
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
}
