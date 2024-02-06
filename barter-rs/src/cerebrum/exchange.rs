use super::event::Event;
use super::exchange_client::{ClientId, ExchangeClient};
use barter_execution::error::ExecutionError;
use barter_execution::model::execution_event::ExecutionRequest;
use barter_execution::model::{AccountEvent, AccountEventKind};
use barter_execution::ExecutionClient;
use barter_integration::model::Exchange;
use futures::stream::{self, BoxStream, SelectAll};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Responsibilities:
/// - Determines best way to action an [`ExecutionRequest`] given the constraints of the exchange.

/// Responsibilities:
/// - Manages every [`ExchangeClient`].
/// - Forwards the request to the appropriate [`ExchangeClient`].
/// - Map InternalClientOrderId to exchange ClientOrderId.

// #[derive(Debug)]
#[allow(missing_debug_implementations)]
pub struct ExchangePortal {
    clients: HashMap<Exchange, Arc<Box<ExchangeClient>>>,
    request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
    event_tx: mpsc::UnboundedSender<Event>,
    account_stream: SelectAll<BoxStream<'static, (Exchange, AccountEventKind)>>,
}

impl ExchangePortal {
    pub async fn init(
        exchanges: HashMap<Exchange, ClientId>,
        request_rx: mpsc::UnboundedReceiver<ExecutionRequest>,
        event_tx: mpsc::UnboundedSender<Event>,
    ) -> Result<Self, ()> {
        // Todo:
        //  - Validate input
        //  - I don't think there is any reason the core would ask for ConnectionStatus, but it would be sent
        //  - Can ExchangePortal act as the Driver? Yes.
        //  - Make ExchangePortal state machine...

        // 1. Store HashMap<Exchange, ClientId> for association & to keep every ClientId(Config)√
        // 2. Spawn tasks for every ExchangeClient √
        // 3. Monitor ConnectionStatus of each task √
        // 4. Re-spawn ExchangeClient if required TODO?

        let mut clients = HashMap::new();

        info!("initializing ExchangePortal {:?}", exchanges);

        let mut streams = Vec::new();

        for (exchange, client_id) in exchanges.into_iter() {
            let client = ExchangeClient::init(client_id).await;
            let stream = client.init_stream().await;
            let ex = exchange.clone();
            if let Some(stream) = stream {
                // map stream to (exchange, stream) tuple
                let stream = stream.map(move |item| (ex.clone(), item)).boxed();
                streams.push(stream);
            }
            clients.insert(exchange, Arc::new(Box::new(client)));
        }

        let account_stream = stream::select_all(streams);

        Ok(Self {
            clients,
            request_rx,
            event_tx,
            account_stream,
        })
    }

    fn send_account_tx(
        event_tx: mpsc::UnboundedSender<Event>,
        exchange: Exchange,
        kind: AccountEventKind,
    ) {
        let account_event = AccountEvent {
            exchange,
            received_time: chrono::Utc::now(),
            kind,
        };
        event_tx
            .send(Event::Account(account_event))
            .expect("Account engine is offline");
    }

    /// Todo:
    ///  - Should be run on it's own OS thread.
    ///  - This may live in Barter... ExchangeClient impls would live here. Order would be in Barter!
    ///  - May need to run enum ExchangeEvent { request, ConnectionStatus } in order to re-spawn clients! -> state machine like Cerebrum!
    pub async fn run(mut self) {
        loop {
            select! {
                // Recieves events from websocket streams and forwards them to
                // Account Engine
                Some((exchange, account_event)) = self.account_stream.next() => {
                    let event_tx = self.event_tx.clone();
                    Self::send_account_tx(event_tx, exchange, account_event);
                },

                // Processes execution requests
                Some(request) = self.request_rx.recv() => {

                    // while let Some(request) = self.request_rx.recv().await {
                    // info!(payload = ?request, "received ExchangeRequest");

                    // Action ExecutionRequest
                    match request {
                        ExecutionRequest::OpenOrders(open_requests) => {
                            open_requests.into_iter().for_each(|open_request| {
                                let exchange = open_request.0;
                                let orders = open_request.1;
                                let client = self.client(&exchange);
                                let tx = self.event_tx.clone();
                                info!("sending OpenOrders ");
                                tokio::spawn(async move {
                                    let open_orders = client.open_orders(orders).await;
                                    let open_orders = remove_error_responses(open_orders);
                                    let account_event = AccountEventKind::OrdersNew(open_orders);
                                    Self::send_account_tx(tx, exchange, account_event);
                                });
                            });
                        }
                        ExecutionRequest::FetchOrdersOpen(exchanges) => {
                            exchanges.into_iter().for_each(|exchange| {
                                let client = self.client(&exchange);
                                let tx = self.event_tx.clone();
                                tokio::spawn(async move {
                                    match client.fetch_orders_open().await {
                                        Ok(orders) => Self::send_account_tx(
                                            tx,
                                            exchange.clone(),
                                            AccountEventKind::OrdersOpen(orders),
                                        ),
                                        Err(e) => error!(error = ?e, "failed to fetch open orders"),
                                    };
                                });
                            });
                        }
                        ExecutionRequest::FetchBalances(exchanges) => {
                            exchanges.into_iter().for_each(|exchange| {
                                let client = self.client(&exchange);
                                let tx = self.event_tx.clone();
                                tokio::spawn(async move {
                                    match client.fetch_balances().await {
                                        Ok(balances) => Self::send_account_tx(
                                            tx,
                                            exchange,
                                            AccountEventKind::Balances(balances),
                                        ),
                                        Err(e) => error!(error = ?e, "failed to fetch balances"),
                                    };
                                });
                            });
                        }
                        ExecutionRequest::CancelOrders(cancel_requests) => {
                            cancel_requests.into_iter().for_each(|cancel_request| {
                                let exchange = cancel_request.0;
                                let orders = cancel_request.1;
                                let client = self.client(&exchange);
                                let tx = self.event_tx.clone();
                                tokio::spawn(async move {
                                    let cancelled_orders = client.cancel_orders(orders).await;
                                    let cancelled_orders = remove_error_responses(cancelled_orders);
                                    let account_event = AccountEventKind::OrdersCancelled(cancelled_orders);
                                    Self::send_account_tx(tx, exchange, account_event);
                                });
                            });
                        }
                        ExecutionRequest::CancelOrdersAll(exchanges) => {
                            exchanges.into_iter().for_each(|exchange| {
                                let client = self.client(&exchange);
                                let tx = self.event_tx.clone();
                                tokio::spawn(async move {
                                    match client.cancel_orders_all().await {
                                        Ok(cancelled_orders) => Self::send_account_tx(
                                            tx,
                                            exchange,
                                            AccountEventKind::OrdersCancelled(cancelled_orders),
                                        ),
                                        Err(e) => error!(error = ?e, "failed to cancel all orders"),
                                    };
                                });
                            });
                        }
                    }
                },
                else => break,
            }
        }
    }

    /// Retrieve the [`ExchangeClient`] associated with the [`Exchange`].
    pub fn client(&self, exchange: &Exchange) -> Arc<Box<ExchangeClient>> {
        self.clients
            .get(exchange)
            .cloned()
            .expect(&format!("cannot retrieve ExchangeClient for {}", exchange))
    }
}

// UTILS
pub fn remove_error_responses<T>(responses: Vec<Result<T, ExecutionError>>) -> Vec<T> {
    responses
        .into_iter()
        .filter_map(|response| match response {
            Ok(response) => Some(response),
            Err(e) => {
                error!(error = ?e, "failed to submit an order");
                None
            }
        })
        .collect::<Vec<T>>()
}

// Todo: client health
// #[derive(Debug, Clone, Copy)]
// pub struct ClientHealth {
//     status: ClientStatus,
//     latency_avg: (),
// }

#[derive(Clone, Copy, Debug)]
pub enum ClientStatus {
    Connected,
    CancelOnly,
    Disconnected,
}
