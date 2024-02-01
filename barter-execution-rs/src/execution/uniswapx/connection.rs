use std::{fmt::Debug, marker::PhantomData};

use bytes::Bytes;

use barter_integration::{
    error::SocketError,
    protocol::http::{
        private::{encoder::HexEncoder, get_default_signer, RequestSigner, Signer},
        rest::{client::RestClient, ApiRequest, QueryParams, RestRequest},
        HttpParser,
    },
};
use chrono::Utc;
use dotenv::dotenv;
use hmac::Hmac;
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;
use tokio::sync::mpsc;
use warp::{Filter, Rejection, Reply};

use crate::{
    error::ExecutionError,
    fill::Decision,
    model::order_event::{OrderEvent, OrderExecutionType, OrderType},
};

use super::requests::{QuoteRequest, QuoteResponse};

#[derive(Debug, Copy, Clone)]
pub enum LiveOrTest {
    Live,
    Test,
}

#[derive(Debug, Copy, Clone)]
pub enum UniswapxApi {
    Spot(LiveOrTest),
    Futures(LiveOrTest),
}

pub type UniswapxInternalClient =
    RestClient<RequestSigner<UniswapxSigner, Hmac<sha2::Sha256>, HexEncoder>, UniswapxParser>;

#[derive(Debug)]
pub struct UniswapxClient {
}

impl UniswapxClient {
    pub fn new(api_type: UniswapxApi) -> UniswapxClient {
        UniswapxClient {}
    }

    pub async fn init(&self) {
        // Define the warp filter for the quote endpoint
        let quote_endpoint = warp::path("quote")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_quote_request);

        // Start the warp server on a specified address and port
        warp::serve(quote_endpoint)
            .run(([127, 0, 0, 1], 8080))
            .await;
    }
}

pub async fn handle_quote_request(request: QuoteRequest) -> Result<impl Reply, Rejection> {
    // WARNING! Just echoing the request atm
    let response = QuoteResponse {
        chainId: request.tokenInChainId,
        amountIn: request.amount.clone(),
        amountOut: request.amount.clone(),
        filler: "HERE's WHERE THE FILLER ADDRES GOES".to_string(),
        requestId: request.requestId,
        swapper: request.swapper,
        tokenIn: request.tokenIn,
        tokenOut: request.tokenOut,
        quoteId: request.quoteId,
    };

    Ok(warp::reply::json(&response))
}

pub(super) fn get_order_side(side: Decision) -> &'static str {
    match side {
        Decision::Long => "BUY",
        Decision::Short => "SELL",
        Decision::CloseLong => "SELL",
        Decision::CloseShort => "BUY",
    }
}

#[derive(Debug)]
pub struct UniswapxSigner {
    pub api_key: String,
    pub timestamp_delta: i64,
}

impl UniswapxSigner {
    pub fn init(api_key: String, timestamp_delta: i64) -> Self {
        Self {
            api_key,
            timestamp_delta,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UniswapxParser;

impl HttpParser for UniswapxParser {
    type ApiError = serde_json::Value;
    type OutputError = ExecutionError;

    fn parse_api_error(
        &self,
        status: StatusCode,
        api_error: Self::ApiError,
        parse_api_error: serde_json::Error,
    ) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        // and combine serde_json::Error with serde_json::Value error
        let error = parse_api_error.to_string() + &api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            }
            _ => ExecutionError::Socket(SocketError::HttpResponse(status, error)),
        }
    }
}

// write test for create_order with mock
#[cfg(test)]
mod tests {
    use warp::{http::StatusCode, Filter};
    use super::super::requests::{QuoteRequest, QuoteResponse};

    #[tokio::test]
    async fn test_quote_endpoint() {
        let quote_endpoint = warp::path("quote")
            .and(warp::post())
            .and(warp::body::json())
            .and_then(|request: QuoteRequest| async move {
                let response = QuoteResponse {
                    chainId: request.tokenInChainId,
                    amountIn: request.amount.clone(),
                    amountOut: request.amount.clone(),
                    filler: "Test filler address".to_string(),
                    requestId: request.requestId,
                    swapper: request.swapper,
                    tokenIn: request.tokenIn,
                    tokenOut: request.tokenOut,
                    quoteId: request.quoteId,
                };
                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            });
    
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
            "quoteId": "test-quote-id"
        }"#;
    
        let response = warp::test::request()
            .method("POST")
            .path("/quote")
            .header("content-type", "application/json")
            .body(request_payload)
            .reply(&quote_endpoint)
            .await;
        
        println!("DATA!: {:?}", response);
        println!("Response: {:?}", response);

        assert_eq!(response.status(), StatusCode::OK);
    }
}
