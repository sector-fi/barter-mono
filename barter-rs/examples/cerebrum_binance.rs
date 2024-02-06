use barter::cerebrum::{
    account::{Account, Accounts},
    event::{Command, Event, EventFeed},
    exchange::ExchangePortal,
    exchange_client::ClientId,
    strategy,
    strategy::IndicatorUpdater,
    Engine,
};
use barter_execution::{
    execution::binance::{
        connection::{BinanceApi, LiveOrTest},
        BinanceConfig,
    },
    model::{
        balance::Balance,
        execution_event::ExecutionRequest,
        order::{Order, OrderKind, RequestCancel, RequestOpen},
        ClientOrderId, Position,
    },
};
use dotenv::dotenv;

use barter_data::{
    event::{DataKind, MarketEvent},
    exchange::{
        binance::{
            futures::{BinanceFuturesUsd, BinanceServerFuturesUsd},
            Binance,
        },
        ExchangeId,
    },
    streams::Streams,
    subscription::{trade::PublicTrades, Subscription},
};
use barter_integration::{
    init_logging,
    model::{
        instrument::{kind::InstrumentKind, Instrument},
        Exchange, Side,
    },
};
use std::ops::Add;
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
// use tracing::info;

struct StrategyExample {
    counter: usize,
    price: Option<f64>,
    // rsi: ta::indicators::RelativeStrengthIndex,
}

impl IndicatorUpdater for StrategyExample {
    fn update_indicators(&mut self, market: &MarketEvent<DataKind>) {
        let price = match &market.kind {
            DataKind::Trade(trade) => trade.price,
            DataKind::Candle(candle) => candle.close,
            _ => panic!("unexpected DataKind"),
        };
        self.price = Some(price);
    }
}

impl strategy::OrderGenerator for StrategyExample {
    fn generate_cancels(
        &mut self,
        _accounts: &Accounts,
    ) -> Option<Vec<(Exchange, Vec<Order<RequestCancel>>)>> {
        None
    }

    fn generate_orders(
        &mut self,
        accounts: &Accounts,
    ) -> Option<Vec<(Exchange, Vec<Order<RequestOpen>>)>> {
        if self.counter > 1 || self.price.is_none() {
            return None;
        }

        let account = accounts.get(&ExchangeId::BinanceFuturesUsd.into());
        let num_open_orders = account.orders_open.len();
        if self.counter > num_open_orders {
            return None;
        }
        println!("accounts {:#?}", accounts);

        let order = order_request_limit(
            Instrument::new("eth", "usdt", InstrumentKind::Perpetual),
            ClientOrderId(uuid::Uuid::new_v4()),
            Side::Buy,
            self.price.unwrap(),
            0.01,
        );

        self.counter += 1;
        Some(vec![(ExchangeId::BinanceFuturesUsd.into(), vec![order])])
    }
}

// Utility for creating an Open Order request
fn order_request_limit<I>(
    instrument: I,
    cid: ClientOrderId,
    side: Side,
    price: f64,
    quantity: f64,
) -> Order<RequestOpen>
where
    I: Into<Instrument>,
{
    Order {
        exchange: ExchangeId::BinanceFuturesUsd.into(),
        instrument: instrument.into(),
        cid,
        side,
        state: RequestOpen {
            kind: OrderKind::Limit,
            price,
            quantity,
        },
    }
}

// Notes:
// - Hard-coded to use one Exchange, Binance
#[tokio::main]
async fn main() {
    dotenv().ok();
    // Initialise structured JSON subscriber
    init_logging();

    // Duration to run before Termination
    let terminate = Duration::from_secs(6000);

    // Central EventFeed: will receive Event::Market, Event::Account & Event::Command
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let feed = EventFeed::new(event_rx);

    // ExchangeCommand Transmitter
    let (exchange_tx, exchange_rx) = mpsc::unbounded_channel();

    // Event Audit Transmitter: Stubbed For Now
    // let (audit_tx, audit_rx) = mpsc::unbounded_channel();
    let audit_tx = ();

    // EventFeed Component: MarketFeed:
    let subscriptions = init_market_feed(event_tx.clone()).await;

    // EventFeed Component: AccountFeed:
    init_account_feed(event_tx.clone(), exchange_rx).await;

    // EventFeed Component: CommandFeed
    init_command_feed(event_tx, terminate);

    let exchange = Exchange::from(ExchangeId::BinanceFuturesUsd);

    // Accounts(HashMap<Exchange, Account>):
    let accounts = init_accounts(exchange, subscriptions);

    // StrategyExample
    let strategy = StrategyExample {
        counter: 0,
        price: None,
        // rsi: ta::indicators::RelativeStrengthIndex::new(14).unwrap(),
    };

    // Build Engine
    let engine = Engine::builder()
        .feed(feed) // Todo: Should builder set this up?
        .accounts(accounts) // Todo: Should builder set this up?
        .exchange_tx(exchange_tx)
        .strategy(strategy)
        .audit_tx(audit_tx)
        .build()
        .expect("failed to build Engine");

    // Run Engine
    std::thread::spawn(move || engine.run());

    // tokio::spawn(async move { engine.run().await })
    //     .await
    //     .unwrap();

    tokio::time::sleep(terminate.add(Duration::from_secs(1))).await
}

async fn init_market_feed<Exchange, Kind>(
    event_tx: mpsc::UnboundedSender<Event>,
) -> Vec<Subscription<Exchange, Kind>>
where
    Vec<Subscription<Exchange, Kind>>:
        FromIterator<Subscription<Binance<BinanceServerFuturesUsd>, PublicTrades>>, // Exchange: Binance<BinanceServerFuturesUsd>,
{
    let subs = vec![(
        BinanceFuturesUsd::default(),
        "eth",
        "usdt",
        InstrumentKind::Perpetual,
        PublicTrades,
    )];

    let mut stream = Streams::<Kind>::builder()
        .subscribe(subs.clone())
        .init()
        .await
        .unwrap();

    let mut market_rx = stream.select(ExchangeId::BinanceFuturesUsd).unwrap();

    tokio::spawn(async move {
        while let Some(trade) = market_rx.recv().await {
            let _ = event_tx.send(Event::Market(MarketEvent::from(trade)));
        }
    });

    // is separate thread necessary here?
    // std::thread::spawn(move || loop {
    //     match market_rx.try_recv() {
    //         Ok(trade) => event_tx
    //             .send(Event::Market(MarketEvent::from(trade)))
    //             .expect("failed to send MarketEvent to EventFeed"),
    //         Err(mpsc::error::TryRecvError::Empty) => {
    //             continue;
    //         }
    //         Err(mpsc::error::TryRecvError::Disconnected) => {
    //             panic!("MarketFeed failed")
    //         }
    //     }
    // });

    // this is needed to init account map
    subs.into_iter()
        .map(|sub| Subscription::from(sub))
        .collect()
}

// Todo:
//  - Will change when we setup the ExchangeClients properly, likely needs Vec<Instrument>
async fn init_account_feed(
    event_tx: mpsc::UnboundedSender<Event>,
    exchange_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
) {
    let mut exchanges = HashMap::new();
    let execution_config = BinanceConfig {
        client_type: BinanceApi::Futures(LiveOrTest::Test),
    };
    exchanges.insert(
        Exchange::from(ExchangeId::BinanceFuturesUsd),
        ClientId::Binance(execution_config),
    );
    let ex_portal = ExchangePortal::init(exchanges, exchange_rx, event_tx)
        .await
        .expect("failed to init ExchangePortal");

    tokio::spawn(async move {
        ex_portal.run().await;
    });

    // alternately we can spawn sync thread
    // std::thread::spawn(move || {
    //     ex_portal.run();
    // });
}

fn init_command_feed(event_tx: mpsc::UnboundedSender<Event>, terminate: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(terminate);
        event_tx.send(Event::Command(Command::Terminate)).unwrap()
    });
}

fn init_accounts<Ex, Kind>(
    exchange: Exchange,
    subscriptions: Vec<Subscription<Ex, Kind>>,
) -> Accounts
where
    Exchange: Eq + std::hash::Hash,
{
    let instruments: Vec<Instrument> = subscriptions
        .into_iter()
        .map(|subscription| subscription.instrument)
        .collect();

    let mut accounts = HashMap::new();
    accounts.insert(exchange, init_account(instruments.clone()));
    Accounts(accounts)
}

fn init_account(instruments: Vec<Instrument>) -> Account {
    let positions = instruments
        .iter()
        .cloned()
        .map(|instrument| (instrument, Position))
        .collect();

    let balances = instruments
        .into_iter()
        .map(|instrument| [instrument.base, instrument.quote])
        .flatten()
        // Todo: Later we will init Balances during Init, so this would be (0.0, 0.0) until exchange update
        .map(|symbol| {
            (
                symbol,
                Balance {
                    total: 1000.0,
                    available: 1000.0,
                },
            )
        })
        .collect();

    Account {
        balances,
        positions,
        orders_in_flight: HashMap::new(),
        orders_open: HashMap::new(),
    }
}
