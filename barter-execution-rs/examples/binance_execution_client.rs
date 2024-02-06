use barter_execution::{
    execution::binance::{
        connection::BinanceApi, connection::LiveOrTest, BinanceConfig, BinanceExecution,
    },
    ExecutionClient,
};
use barter_integration::init_logging;
// use barter_integration::init_logging;
use dotenv::dotenv;

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();
    let _binance_execution_client: BinanceExecution = ExecutionClient::init(BinanceConfig {
        client_type: BinanceApi::Futures(LiveOrTest::Test),
    })
    .await;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
