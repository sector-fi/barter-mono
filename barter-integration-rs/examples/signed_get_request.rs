use barter_integration::{
    error::SocketError,
    metric::Tag,
    model::instrument::symbol::Symbol,
    protocol::http::{
        private::{encoder::HexEncoder, RequestSigner, Signer},
        rest::{client::RestClient, RestRequest},
        HttpParser,
    },
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use hmac::{digest::KeyInit, Hmac};
use reqwest::{RequestBuilder, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc;

struct FtxSigner {
    api_key: String,
}

// Configuration required to sign every Ftx `RestRequest`
struct FtxSignConfig<'a> {
    api_key: &'a str,
    time: DateTime<Utc>,
    method: reqwest::Method,
    path: &'static str,
}

impl Signer for FtxSigner {
    type Config<'a> = FtxSignConfig<'a> where Self: 'a;

    fn config<'a, Request>(
        &'a self,
        request: &Request,
        builder: RequestBuilder,
    ) -> Result<(Self::Config<'a>, RequestBuilder), SocketError>
    where
        Request: RestRequest,
    {
        Ok((
            FtxSignConfig {
                api_key: self.api_key.as_str(),
                time: Utc::now(),
                method: request.method(),
                path: request.path(),
            },
            builder,
        ))
    }

    fn bytes_to_sign<'a>(config: &Self::Config<'a>) -> Bytes {
        Bytes::copy_from_slice(
            format!("{}{}{}", config.time, config.method, config.path).as_bytes(),
        )
    }

    fn build_signed_request<'a>(
        config: Self::Config<'a>,
        builder: RequestBuilder,
        signature: String,
    ) -> Result<reqwest::Request, SocketError> {
        // Add Ftx required Headers & build reqwest::Request
        builder
            .header("FTX-KEY", config.api_key)
            .header("FTX-TS", &config.time.timestamp_millis().to_string())
            .header("FTX-SIGN", &signature)
            .build()
            .map_err(SocketError::from)
    }
}

struct FtxParser;

impl HttpParser for FtxParser {
    type ApiError = serde_json::Value;
    type OutputError = ExecutionError;

    fn parse_api_error(
        &self,
        status: StatusCode,
        api_error: Self::ApiError,
        _parse_api_error: serde_json::Error,
    ) -> Self::OutputError {
        // For simplicity, use serde_json::Value as Error and extract raw String for parsing
        let error = api_error.to_string();

        // Parse Ftx error message to determine custom ExecutionError variant
        match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            }
            _ => ExecutionError::Socket(SocketError::HttpResponse(status, error)),
        }
    }
}

#[derive(Debug, Error)]
enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError),
}

#[derive(Debug)]
pub struct FetchBalancesRequest;

impl RestRequest for FetchBalancesRequest {
    type Response = FetchBalancesResponse; // Define Response type
    type Body = (); // FetchBalances does not require any Body

    fn path(&self) -> &'static str {
        "/api/wallet/balances"
    }

    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }

    fn metric_tag(&self) -> Tag {
        Tag::new("method", "fetch_balances")
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct FetchBalancesResponse {
    success: bool,
    result: Vec<FtxBalance>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct FtxBalance {
    #[serde(rename = "coin")]
    symbol: Symbol,
    total: f64,
}

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    // Construct Metric channel to send Http execution metrics over
    let (http_metric_tx, _http_metric_rx) = mpsc::unbounded_channel();

    // HMAC-SHA256 encoded account API secret used for signing private http requests
    let mac: Hmac<sha2::Sha256> = Hmac::new_from_slice("api_secret".as_bytes()).unwrap();

    // Build Ftx configured RequestSigner for signing http requests with hex encoding
    let request_signer = RequestSigner::new(
        FtxSigner {
            api_key: "api_key".to_string(),
        },
        mac,
        HexEncoder,
    );

    // Build RestClient with Ftx configuration
    let rest_client = RestClient::new(
        "https://ftx.com".to_string(),
        http_metric_tx,
        request_signer,
        FtxParser,
    );

    // Fetch Result<FetchBalancesResponse, ExecutionError>
    let _response = rest_client.execute(FetchBalancesRequest).await;
}
