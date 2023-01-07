use super::SubKind;
use crate::{
    event::{MarketIter, Market},
    exchange::ExchangeId,
};
use barter_integration::model::{Exchange, Instrument};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use barter_integration::error::SocketError;

// Todo:
// - Remove un-required fields from OrderBookL1 & OrderBook (ie/ update fields)

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields level 1 [`OrderBook`]
/// [`Market`](crate::model::Market) events.
///
/// Level 1 refers to the best non-aggregated bid and ask [`Level`] on each side of the
/// [`OrderBook`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct OrderBooksL1;

impl SubKind for OrderBooksL1 {
    type Event = OrderBookL1;
}

/// Normalised Barter [`OrderBookL1`] snapshot containing the latest best bid and ask.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderBookL1 {
    pub last_update_time: DateTime<Utc>,
    pub last_update_id: u64,
    pub best_bid: Level,
    pub best_ask: Level,
}

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields level 2 [`OrderBook`]
/// [`Market`](crate::model::Market) events.
///
/// Level 2 refers to the [`OrderBook`] aggregated by price.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct OrderBooksL2;

impl SubKind for OrderBooksL2 {
    type Event = OrderBook;
}

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields level 3 [`OrderBook`]
/// [`Market`](crate::model::Market) events.
///
/// Level 3 refers to the non-aggregated [`OrderBook`]. This is a direct replication of the exchange
/// [`OrderBook`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct OrderBooksL3;

impl SubKind for OrderBooksL3 {
    type Event = OrderBook;
}

/// Normalised Barter [`OrderBook`] snapshot.
#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Deserialize, Serialize)]
pub struct OrderBook {
    pub last_update_time: DateTime<Utc>,
    pub bids: OrderBookSide,
    pub asks: OrderBookSide,
}

/// Normalised Barter [`Level`]s for one [`Side`] of the [`OrderBook`].
pub struct OrderBookSide(pub Vec<Level>);

impl<L> FromIterator<L> for OrderBookSide
where
    Level: From<L>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = L>,
    {
        Self(iter.into_iter().map(Level::from).collect())
    }
}

impl OrderBookSide {
    pub fn upsert<Iter>(&mut self, levels: Iter) -> Result<(), SocketError> {
        todo!()
    }
}

/// Normalised Barter OrderBook [`Level`].
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Level {
    pub price: f64,
    pub amount: f64,
}

impl<T> From<(T, T)> for Level
where
    T: Into<f64>,
{
    fn from((price, amount): (T, T)) -> Self {
        Self::new(price, amount)
    }
}

impl Level {
    pub fn new<T>(price: T, amount: T) -> Self
    where
        T: Into<f64>,
    {
        Self {
            price: price.into(),
            amount: amount.into(),
        }
    }
}

impl From<(ExchangeId, Instrument, OrderBook)> for MarketIter<OrderBook> {
    fn from((exchange_id, instrument, book): (ExchangeId, Instrument, OrderBook)) -> Self {
        Self(vec![Ok(Market {
            exchange_time: book.last_update_time,
            received_time: Utc::now(),
            exchange: Exchange::from(exchange_id),
            instrument,
            event: book,
        })])
    }
}
