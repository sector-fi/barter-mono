use super::SubKind;
use barter_integration::model::instrument::Instrument;
use serde::{Deserialize, Serialize};

/// Barter [`Subscription`](super::Subscription) [`SubKind`] that yields [`RfqRequest`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct RfqRequests;

impl SubKind for RfqRequests {
    type Event = RfqRequest;
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct RfqRequest {
    pub instrument: Instrument,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
    pub token_in_chain_id: u32,
    pub token_out_chain_id: u32,
    pub swapper: String,
    pub token_in: String,
    pub token_out: String,
    pub ask: f64,
    pub ask_raw: String,
}
