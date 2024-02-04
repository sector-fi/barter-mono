use std::collections::HashMap;

use async_trait::async_trait;
use barter_integration::model::{
    instrument::{symbol::Symbol, Instrument},
    Exchange,
};
use chrono::Utc;
use tokio::sync::mpsc;

use crate::{
    error::ExecutionError,
    fill::{Fees, FillEvent},
    model::{
        balance::SymbolBalance,
        execution_event::ExchangeRequest,
        order::{Cancelled, Open, Order, RequestCancel, RequestOpen},
        order_event::OrderEvent,
        AccountEvent,
    },
    ExecutionClient, ExecutionId,
};

use self::{
    connection::{UniswapxApi, UniswapxClient},
    // requests::{FutOrderResponse, FUT_BALANCES_REQUEST},
};

pub mod connection;
pub mod requests;

// Simulated [`ExecutionClient`] implementation that integrates with the Barter
/// [`SimulatedExchange`](super::exchange::SimulatedExchange).
#[derive(Debug)]
pub struct UniswapxExecution<Event> {
    client: UniswapxClient,
    instruments_map: HashMap<UniswapxPair, Instrument>,
    // client_type: BinanceApi,
    pub event_tx: mpsc::UnboundedSender<Event>,
    pub order_tx: mpsc::UnboundedSender<ExchangeRequest>,
}

/// Config for initializing a [`SimulatedExecution`] instance.
#[derive(Debug)]
pub struct UniswapxConfig {
    client: UniswapxClient,
    instruments: Vec<Instrument>,
    order_tx: mpsc::UnboundedSender<ExchangeRequest>,
}

#[async_trait]
impl<Event> ExecutionClient for UniswapxExecution<Event> 
where
    Event: Send + From<AccountEvent>,
{
    const CLIENT: ExecutionId = ExecutionId::Simulated;
    type Config = UniswapxConfig;
    type Event = Event;

    async fn init(config: Self::Config, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        let client = UniswapxClient::new();
        Self {
            client,
            instruments_map: Self::instruments_map(config.instruments),
            event_tx,
            order_tx: config.order_tx,
        }
    }

    // fn request_tx(&self) -> mpsc::UnboundedSender<ExchangeRequest> {
    //     // self.order_tx.clone()
    //     todo!()
    // }

    fn event_tx(&self) -> &mpsc::UnboundedSender<Event> {
        &self.event_tx
    }

    fn exchange(&self) -> Exchange {
        Exchange::from(ExecutionId::Uniswapx)
    }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        todo!()
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        todo!()
    }

    async fn open_orders(
        &self,
        _open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        todo!()
    }

    async fn cancel_orders(
        &self,
        _cancel_requests: Vec<Order<RequestCancel>>,
    ) -> Vec<Result<Order<Cancelled>, ExecutionError>> {
        todo!()
    }

    async fn cancel_orders_all(&self) -> Result<Vec<Order<Cancelled>>, ExecutionError> {
        todo!()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct UniswapxPair(String);

impl UniswapxPair {
    pub fn new(base: &Symbol, quote: &Symbol) -> Self {
        Self(format!("{base}{quote}").to_uppercase())
    }
}

impl<Event> UniswapxExecution<Event> {
    fn instruments_map(instruments: Vec<Instrument>) -> HashMap<UniswapxPair, Instrument> {
        instruments
            .into_iter()
            .map(|instrument| {
                (
                    UniswapxPair::new(&instrument.base, &instrument.quote),
                    instrument,
                )
            })
            .collect()
    }
}
