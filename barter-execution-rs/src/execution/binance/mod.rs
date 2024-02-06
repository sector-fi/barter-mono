use async_trait::async_trait;
use barter_integration::model::{instrument::symbol::Symbol, Exchange};
use futures::future::join_all;
use tracing::{error, info};

use crate::{
    error::ExecutionError,
    model::{
        balance::SymbolBalance,
        order::{self, Cancelled, Open, Order, OrderId, RequestCancel, RequestOpen},
    },
    ExecutionClient, ExecutionId,
};

use self::{
    connection::{BinanceApi, BinanceClient},
    requests::{FutOrderResponse, FUT_BALANCES_REQUEST},
    websocket::init_listener,
};

pub mod connection;
pub mod requests;
pub mod websocket;

/// Binance [`ExecutionClient`] implementation that integrates with the Barter
#[derive(Debug)]
pub struct BinanceExecution {
    client: BinanceClient,
    // client_type: BinanceApi,
}

/// Config for initializing a [`BinanceExecution`] instance.
#[derive(Debug, Clone, Copy)]
pub struct BinanceConfig {
    pub client_type: BinanceApi,
}

#[async_trait]
impl ExecutionClient for BinanceExecution {
    type Config = BinanceConfig;

    fn exchange(&self) -> Exchange {
        Exchange::from(ExecutionId::Simulated)
    }

    async fn init(config: Self::Config) -> Self {
        let client = BinanceClient::new(config.client_type);
        let url = BinanceClient::get_url(config.client_type);
        let (api_key, _) = BinanceClient::get_key_secret(config.client_type);
        init_listener(&api_key, url).await;
        Self {
            client,
            // client_type: config.client_type,
        }
    }

    async fn fetch_orders_open(&self) -> Result<Vec<Order<Open>>, ExecutionError> {
        todo!()
    }

    async fn fetch_balances(&self) -> Result<Vec<SymbolBalance>, ExecutionError> {
        match self.client.send(FUT_BALANCES_REQUEST).await {
            Ok(response) => {
                println!("{:#?}", response);
                return Ok(<Vec<SymbolBalance>>::from(response));
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(e);
            }
        }
    }

    async fn open_orders(
        &self,
        open_requests: Vec<Order<RequestOpen>>,
    ) -> Vec<Result<Order<Open>, ExecutionError>> {
        let mut tasks = Vec::new();
        for open_request in open_requests {
            let client = self.client.clone();
            let task = tokio::spawn(async move {
                let res = client.open_order::<FutOrderResponse>(&open_request).await;
                match res {
                    Ok(res) => Ok(Order::<Open>::from((
                        OrderId::from(res.orderId),
                        open_request,
                    ))),
                    // TODO figure out why it failed
                    Err(e) => {
                        info!("{:?}", e);
                        Err(e)
                    }
                }
            });
            tasks.push(task);
        }

        join_all(tasks)
            .await
            .into_iter()
            .map(|res| res.unwrap())
            .collect()
    }

    // TODO batch orders?

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
pub struct BinancePair(String);

impl BinancePair {
    pub fn new(base: &Symbol, quote: &Symbol) -> Self {
        Self(format!("{base}{quote}").to_uppercase())
    }
}
