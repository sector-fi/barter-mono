use barter_execution::{
    execution::binance::{
        connection::BinanceApi,
        connection::{BinanceClient, LiveOrTest},
        requests::FutOrderResponse,
    },
    model::{
        order::{Order, OrderKind, RequestOpen},
        ClientOrderId,
    },
    ExecutionId,
};
use barter_integration::model::{
    instrument::{kind::InstrumentKind, Instrument},
    Exchange, Side,
};
use uuid::Uuid;

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    let order = Order {
        exchange: Exchange::from(ExecutionId::Binance),
        instrument: Instrument::from(("eth", "usdt", InstrumentKind::Perpetual)),
        state: RequestOpen {
            kind: OrderKind::Limit,
            price: 10000.0,
            quantity: 0.001,
        },
        side: Side::Buy,
        cid: ClientOrderId(Uuid::new_v4()),
    };

    let rest_client = BinanceClient::new(BinanceApi::Futures(LiveOrTest::Test));

    // Build RestClient with Binance configuration
    match rest_client.open_order::<FutOrderResponse>(&order).await {
        Ok(response) => println!("{:#?}", response),
        Err(e) => println!("{:?}", e),
    }
}
