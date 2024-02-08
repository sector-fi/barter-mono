// Define your data structures for request and response
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct QuoteRequest {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "tokenInChainId")]
    pub token_in_chain_id: u32,
    #[serde(rename = "tokenOutChainId")]
    pub token_out_chain_id: u32,
    pub swapper: String,
    #[serde(rename = "tokenIn")]
    pub token_in: String,
    #[serde(rename = "tokenOut")]
    pub token_out: String,
    pub amount: String,
    pub type_: u32,
    #[serde(rename = "quoteId")]
    pub quote_id: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct QuoteResponse {
    #[serde(rename = "chainId")]
    pub chain_id: u32,
    #[serde(rename = "amountIn")]
    pub amount_in: String,
    #[serde(rename = "amountOut")]
    pub amount_out: String,
    pub filler: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub swapper: String,
    #[serde(rename = "tokenIn")]
    pub token_in: String,
    #[serde(rename = "tokenOut")]
    pub token_out: String,
    #[serde(rename = "quoteId")]
    pub quote_id: String,
}
