// Define your data structures for request and response
#[derive(Debug, serde::Deserialize)]
pub struct QuoteRequest {
    pub requestId: String,
    pub tokenInChainId: u32,
    pub tokenOutChainId: u32,
    pub swapper: String,
    pub tokenIn: String,
    pub tokenOut: String,
    pub amount: String,
    pub type_: u32,
    pub quoteId: String,
}

#[derive(Debug, serde::Serialize)]
pub struct QuoteResponse {
    pub chainId: u32,
    pub amountIn: String,
    pub amountOut: String,
    pub filler: String,
    pub requestId: String,
    pub swapper: String,
    pub tokenIn: String,
    pub tokenOut: String,
    pub quoteId: String,
}