use barter::cerebrum::{
    account::{Account, Accounts, Position},
    event::{Command, Event, EventFeed},
    exchange::ExchangePortal,
    exchange_client::ClientId,
    strategy,
    strategy::IndicatorUpdater,
    Engine,
};
use barter_execution::{
    fill::Fees,
    model::{
        balance::Balance,
        execution_event::ExecutionRequest,
        order::{Order, OrderKind, RequestCancel, RequestOpen},
        ClientOrderId,
    },
    simulated::{execution::SimulationConfig, util::run_default_exchange, SimulatedEvent},
    ExecutionId,
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

struct StrategyExample {
    counter: usize,
    // rsi: ta::indicators::RelativeStrengthIndex,
}

impl IndicatorUpdater for StrategyExample {
    fn update_indicators(&mut self, market: &MarketEvent<DataKind>) {
        match &market.kind {
            DataKind::Trade(trade) => trade.price,
            DataKind::Candle(candle) => candle.close,
            _ => panic!("unexpected DataKind"),
        };
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
        return None;
        if self.counter > 10 {
            return None;
        }
        let sim_acc = accounts.get(&Exchange::from(ExecutionId::Simulated));
        let num_open_orders = sim_acc.orders_open.len();
        if self.counter > num_open_orders {
            return None;
        }
        println!("accounts {:#?}", sim_acc.orders_open.len());

        let order = order_request_limit(
            Instrument::new("btc", "usdt", InstrumentKind::Perpetual),
            ClientOrderId(uuid::Uuid::new_v4()),
            Side::Buy,
            10000.0,
            0.001,
        );

        self.counter += 1;
        Some(vec![(Exchange::from(ExecutionId::Simulated), vec![order])])
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
        exchange: Exchange::from(ExecutionId::Simulated),
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

    let (event_simulated_tx, event_simulated_rx) = mpsc::unbounded_channel();
    let (execution_tx, _execution_rx) = mpsc::unbounded_channel();

    // Build SimulatedExchange & run on it's own Tokio task
    tokio::spawn(run_default_exchange(execution_tx, event_simulated_rx));

    // EventFeed Component: AccountFeed:
    init_account_feed(event_tx.clone(), exchange_rx, event_simulated_tx).await;

    // EventFeed Component: CommandFeed
    init_command_feed(event_tx, terminate);

    let exchange: Exchange = Exchange::from(ExchangeId::BinanceFuturesUsd);

    // Accounts(HashMap<Exchange, Account>):
    let accounts = init_accounts(exchange, subscriptions);

    // StrategyExample
    let strategy = StrategyExample {
        counter: 0,
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
    let subs = vec![
        (
            BinanceFuturesUsd::default(),
            "btc",
            "usdt",
            InstrumentKind::Perpetual,
            PublicTrades,
        ),
        (
            BinanceFuturesUsd::default(),
            "eth",
            "usdt",
            InstrumentKind::Perpetual,
            PublicTrades,
        ),
        (
            BinanceFuturesUsd::default(),
            "xrp",
            "usdt",
            InstrumentKind::Perpetual,
            PublicTrades,
        ),
    ];

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
    event_simulated_tx: mpsc::UnboundedSender<SimulatedEvent>,
) {
    let mut exchanges = HashMap::new();
    let sim_config = SimulationConfig {
        simulated_fees_pct: Fees {
            exchange: 0.1,
            slippage: 0.05,
            network: 0.0,
        },
        request_tx: event_simulated_tx,
    };
    exchanges.insert(ExecutionId::Simulated, ClientId::Simulated(sim_config));
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
    // we need to init instruments for simulated exchange
    accounts.insert(
        Exchange::from(ExecutionId::Simulated),
        init_account(instruments),
    );
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
