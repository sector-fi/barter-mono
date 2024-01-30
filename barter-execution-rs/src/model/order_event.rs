use barter_integration::model::{instrument::Instrument, Exchange};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::fill::{Decision, MarketMeta};

/// All errors generated in the barter::portfolio module.
#[derive(Copy, Clone, Error, Debug)]
pub enum OrderEventError {
    #[error("Failed to build struct due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),
}

/// Orders are generated by the portfolio and details work to be done by an Execution handler to
/// open a trade.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct OrderEvent {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub exchange: Exchange,
    pub instrument: Instrument,
    /// Metadata propagated from source MarketEvent
    pub market_meta: MarketMeta,
    /// LONG, CloseLong, SHORT or CloseShort
    pub decision: Decision,
    /// +ve or -ve Quantity depending on Decision
    pub quantity: f64,
    /// MARKET, LIMIT etc
    pub order_type: OrderType,
}

impl OrderEvent {
    pub const ORGANIC_ORDER: &'static str = "Order";
    pub const FORCED_EXIT_ORDER: &'static str = "OrderForcedExit";

    /// Returns a OrderEventBuilder instance.
    pub fn builder() -> OrderEventBuilder {
        OrderEventBuilder::new()
    }
}

/// Type of order the portfolio wants the execution::handler to place.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrderType {
    Market,
    Limit {
        price: f64,
        execution_type: OrderExecutionType,
    },
    Bracket,
    StopLoss {
        stop_price: f64,
    },
    TrailingStop {
        trailing_delta: f64,
        stop_price: Option<f64>,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum OrderExecutionType {
    None = 0,
    MakerOnly = 1,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Market
    }
}

/// Builder to construct OrderEvent instances.
#[derive(Debug, Default)]
pub struct OrderEventBuilder {
    pub time: Option<DateTime<Utc>>,
    pub exchange: Option<Exchange>,
    pub instrument: Option<Instrument>,
    pub market_meta: Option<MarketMeta>,
    pub decision: Option<Decision>,
    pub quantity: Option<f64>,
    pub order_type: Option<OrderType>,
}

impl OrderEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(self, value: DateTime<Utc>) -> Self {
        Self {
            time: Some(value),
            ..self
        }
    }

    pub fn exchange(self, value: Exchange) -> Self {
        Self {
            exchange: Some(value),
            ..self
        }
    }

    pub fn instrument(self, value: Instrument) -> Self {
        Self {
            instrument: Some(value),
            ..self
        }
    }

    pub fn market_meta(self, value: MarketMeta) -> Self {
        Self {
            market_meta: Some(value),
            ..self
        }
    }

    pub fn decision(self, value: Decision) -> Self {
        Self {
            decision: Some(value),
            ..self
        }
    }

    pub fn quantity(self, value: f64) -> Self {
        Self {
            quantity: Some(value),
            ..self
        }
    }

    pub fn order_type(self, value: OrderType) -> Self {
        Self {
            order_type: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<OrderEvent, OrderEventError> {
        Ok(OrderEvent {
            id: Uuid::new_v4(),
            time: self
                .time
                .ok_or(OrderEventError::BuilderIncomplete("time"))?,
            exchange: self
                .exchange
                .ok_or(OrderEventError::BuilderIncomplete("exchange"))?,
            instrument: self
                .instrument
                .ok_or(OrderEventError::BuilderIncomplete("instrument"))?,
            market_meta: self
                .market_meta
                .ok_or(OrderEventError::BuilderIncomplete("market_meta"))?,
            // .ok_or(OrderEventError::BuilderIncomplete("market_meta"))?,
            decision: self
                .decision
                .ok_or(OrderEventError::BuilderIncomplete("decision"))?,
            quantity: self
                .quantity
                .ok_or(OrderEventError::BuilderIncomplete("quantity"))?,
            order_type: self
                .order_type
                .ok_or(OrderEventError::BuilderIncomplete("order_type"))?,
        })
    }
}
