use barter_data::event::{DataKind, MarketEvent};
use barter_execution::fill::{Decision, MarketMeta};
use barter_integration::model::{instrument::Instrument, Exchange, Market};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Barter example RSI strategy [`SignalGenerator`] implementation.
pub mod example;

pub mod mm;

pub mod glft;

/// May generate an advisory [`Signal`] as a result of analysing an input [`MarketEvent`].
pub trait SignalGenerator {
    /// Optionally return a [`Signal`] given input [`MarketEvent`].
    fn generate_signal(&mut self, market: &MarketEvent<DataKind>) -> Option<Signal>;
    fn on_market_feed_finished(&mut self) {}
}

/// Advisory [`Signal`] for a [`Market`] detailing the [`SignalStrength`] associated with each
/// possible [`Decision`]. Interpreted by an [`OrderGenerator`](crate::portfolio::OrderGenerator).
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Signal {
    pub time: DateTime<Utc>,
    pub exchange: Exchange,
    pub instrument: Instrument,
    pub signals: HashMap<Decision, SignalStrength>,
    /// Metadata propagated from the [`MarketEvent`] that yielded this [`Signal`].
    pub market_meta: MarketMeta,
}

/// Strength of an advisory [`Signal`] decision produced by [`SignalGenerator`] strategy.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct SignalStrength(pub f64);

/// Force exit Signal produced after an [`Engine`](crate::engine::Engine) receives a
/// [`Command::ExitPosition`](crate::engine::Command) from an external source.
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct SignalForceExit {
    pub time: DateTime<Utc>,
    pub exchange: Exchange,
    pub instrument: Instrument,
}

impl<M> From<M> for SignalForceExit
where
    M: Into<Market>,
{
    fn from(market: M) -> Self {
        let market = market.into();
        Self::new(market.exchange, market.instrument)
    }
}

impl SignalForceExit {
    pub const FORCED_EXIT_SIGNAL: &'static str = "SignalForcedExit";

    /// Constructs a new [`Self`] using the configuration provided.
    pub fn new<E, I>(exchange: E, instrument: I) -> Self
    where
        E: Into<Exchange>,
        I: Into<Instrument>,
    {
        Self {
            time: Utc::now(),
            exchange: exchange.into(),
            instrument: instrument.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_decision_is_long() {
        let decision = Decision::Long;
        assert_eq!(decision.is_long(), true)
    }

    #[test]
    fn should_return_decision_is_not_long() {
        let decision = Decision::Short;
        assert_eq!(decision.is_long(), false)
    }

    #[test]
    fn should_return_decision_is_short() {
        let decision = Decision::Short;
        assert_eq!(decision.is_short(), true)
    }

    #[test]
    fn should_return_decision_is_not_short() {
        let decision = Decision::Long;
        assert_eq!(decision.is_short(), false)
    }

    #[test]
    fn should_return_decision_is_entry() {
        let decision = Decision::Long;
        assert_eq!(decision.is_entry(), true)
    }

    #[test]
    fn should_return_decision_is_not_entry() {
        let decision = Decision::CloseLong;
        assert_eq!(decision.is_entry(), false)
    }

    #[test]
    fn should_return_decision_is_exit() {
        let decision = Decision::CloseShort;
        assert_eq!(decision.is_exit(), true)
    }

    #[test]
    fn should_return_decision_is_not_exit() {
        let decision = Decision::Long;
        assert_eq!(decision.is_exit(), false)
    }
}
