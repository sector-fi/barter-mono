use crate::model::{
    order::OrderId,
    trade::{SymbolFees, Trade, TradeId},
    AccountEventKind,
};

use super::BinanceFuturesEventType;
use barter_integration::model::{
    instrument::{kind::InstrumentKind, symbol::Symbol, Instrument},
    PerpSide, Side,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// enums for the order status
/// Order Type
/// MARKET
/// LIMIT
/// STOP
/// TAKE_PROFIT
/// LIQUIDATION
///
/// Execution Type
/// NEW
/// CANCELED
/// CALCULATED - Liquidation Execution
/// EXPIRED
/// TRADE
/// AMENDMENT - Order Modified
///
/// Order Status
/// NEW
/// PARTIALLY_FILLED
/// FILLED
/// CANCELED
/// EXPIRED
/// EXPIRED_IN_MATCH
///
/// Time in force
/// GTC
/// IOC
/// FOK
/// GTX
///
/// Working Type
/// MARK_PRICE
/// CONTRACT_PRICE

//// Order status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinanceFutOrderStatus {
    #[serde(rename = "NEW")]
    New,
    #[serde(rename = "PARTIALLY_FILLED")]
    PartiallyFilled,
    #[serde(rename = "FILLED")]
    Filled,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "EXPIRED_IN_MATCH")]
    ExpiredInMatch,
}

/// Execution type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinFutExecutionType {
    #[serde(rename = "NEW")]
    New,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "CALCULATED")]
    Calculated,
    #[serde(rename = "EXPIRED")]
    Expired,
    #[serde(rename = "TRADE")]
    Trade,
    #[serde(rename = "AMENDMENT")]
    Amendment,
}

/// Order type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinFutOrderType {
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "STOP")]
    Stop,
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit,
    #[serde(rename = "LIQUIDATION")]
    Liquidation,
}

//// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) AccountUpdate messages.
///
/// ### Raw Payload Examples
/// See docs: <https://binance-docs.github.io/apidocs/futures/en/#event-balance-and-position-update>
/// ```json
/// {
///   "e":"ORDER_TRADE_UPDATE",     // Event Type
///   "E":1568879465651,            // Event Time
///   "T":1568879465650,            // Transaction Time
///   "o":{                             
///     "s":"BTCUSDT",              // Symbol
///     "c":"TEST",                 // Client Order Id
///       // special client order id:
///       // starts with "autoclose-": liquidation order
///       // "adl_autoclose": ADL auto close order
///       // "settlement_autoclose-": settlement order for delisting or delivery
///     "S":"SELL",                 // Side
///     "o":"TRAILING_STOP_MARKET", // Order Type
///     "f":"GTC",                  // Time in Force
///     "q":"0.001",                // Original Quantity
///     "p":"0",                    // Original Price
///     "ap":"0",                   // Average Price
///     "sp":"7103.04",             // Stop Price. Please ignore with TRAILING_STOP_MARKET order
///     "x":"NEW",                  // Execution Type
///     "X":"NEW",                  // Order Status
///     "i":8886774,                // Order Id
///     "l":"0",                    // Order Last Filled Quantity
///     "z":"0",                    // Order Filled Accumulated Quantity
///     "L":"0",                    // Last Filled Price
///     "N":"USDT",                 // Commission Asset, will not push if no commission
///     "n":"0",                    // Commission, will not push if no commission
///     "T":1568879465650,          // Order Trade Time
///     "t":0,                      // Trade Id
///     "b":"0",                    // Bids Notional
///     "a":"9.91",                 // Ask Notional
///     "m":false,                  // Is this trade the maker side?
///     "R":false,                  // Is this reduce only
///     "wt":"CONTRACT_PRICE",      // Stop Price Working Type
///     "ot":"TRAILING_STOP_MARKET",// Original Order Type
///     "ps":"LONG",                // Position Side
///     "cp":false,                 // If Close-All, pushed with conditional order
///     "AP":"7476.89",             // Activation Price, only puhed with TRAILING_STOP_MARKET order
///     "cr":"5.0",                 // Callback Rate, only puhed with TRAILING_STOP_MARKET order
///     "pP": false,                // If price protection is turned on
///     "si": 0,                    // ignore
///     "ss": 0,                    // ignore
///     "rp":"0",                   // Realized Profit of the trade
///     "V":"EXPIRE_TAKER",         // STP mode
///     "pm":"OPPONENT",            // Price match mode
///     "gtd":0                     // TIF GTD order auto cancel time
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFutOrderUpdate {
    /// Event type.
    #[serde(rename = "e")]
    pub event_type: BinanceFuturesEventType,
    /// Event time.
    #[serde(
        rename = "E",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub event_time: DateTime<Utc>,
    /// Transaction time.
    #[serde(
        rename = "T",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub transaction_time: DateTime<Utc>,
    /// Order.
    #[serde(rename = "o")]
    pub order: BinanceFutOrder,
}

/// Order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFutOrder {
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: Symbol,
    /// Client order id.
    #[serde(rename = "c")]
    pub client_order_id: String,
    /// Side.
    #[serde(rename = "S")]
    pub side: Side,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: BinFutOrderType,
    /// Time in force.
    #[serde(rename = "f")]
    pub time_in_force: String,
    /// Original quantity.
    #[serde(rename = "q", deserialize_with = "barter_integration::de::de_str")]
    pub original_quantity: f64,
    /// Original price.
    #[serde(rename = "p", deserialize_with = "barter_integration::de::de_str")]
    pub original_price: f64,
    /// Average price.
    #[serde(rename = "ap", deserialize_with = "barter_integration::de::de_str")]
    pub average_price: f64,
    /// Stop price.
    #[serde(rename = "sp", deserialize_with = "barter_integration::de::de_str")]
    pub stop_price: f64,
    /// Execution type.
    #[serde(rename = "x")]
    pub execution_type: BinFutExecutionType,
    /// Order status.
    #[serde(rename = "X")]
    pub order_status: BinanceFutOrderStatus,
    /// Order id.
    #[serde(rename = "i")]
    pub order_id: u64,
    /// Order last filled quantity.
    #[serde(rename = "l", deserialize_with = "barter_integration::de::de_str")]
    pub order_last_filled_quantity: f64,
    /// Order filled accumulated quantity.
    #[serde(rename = "z", deserialize_with = "barter_integration::de::de_str")]
    pub order_filled_accumulated_quantity: f64,
    /// Last filled price.
    #[serde(rename = "L", deserialize_with = "barter_integration::de::de_str")]
    pub last_filled_price: f64,
    /// Commission asset.
    #[serde(default, rename = "N")]
    pub commission_asset: Option<Symbol>,
    /// Commission.
    #[serde(
        default,
        rename = "n",
        deserialize_with = "barter_integration::de::de_option_f64"
    )]
    pub commission: Option<f64>,
    /// Order trade time.
    #[serde(
        rename = "T",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub order_trade_time: DateTime<Utc>,
    /// Trade id.
    #[serde(rename = "t")]
    pub trade_id: u64,
    /// Bids notional.
    #[serde(rename = "b", deserialize_with = "barter_integration::de::de_str")]
    pub bids_notional: f64,
    /// Ask notional.
    #[serde(rename = "a", deserialize_with = "barter_integration::de::de_str")]
    pub ask_notional: f64,
    /// Is this trade the maker side?
    #[serde(rename = "m")]
    pub is_maker_side: bool,
    /// Is this reduce only.
    #[serde(rename = "R")]
    pub is_reduce_only: bool,
    #[serde(rename = "cp")]
    pub is_close_all: Option<bool>,
    #[serde(
        default,
        rename = "AP",
        deserialize_with = "barter_integration::de::de_option_f64"
    )]
    pub activation_price: Option<f64>,
    #[serde(
        default,
        rename = "cr",
        deserialize_with = "barter_integration::de::de_option_f64"
    )]
    pub callback_rate: Option<f64>,
    #[serde(rename = "pP")]
    pub is_price_protection: Option<bool>,
    #[serde(
        rename = "rp",
        default,
        deserialize_with = "barter_integration::de::de_option_f64"
    )]
    pub realized_profit: Option<f64>,
    #[serde(rename = "V")]
    pub stp_mode: String,
    #[serde(rename = "pm")]
    pub price_match_mode: String,
    #[serde(rename = "gtd")]
    pub tif_gtd_order_auto_cancel_time: i64,
    #[serde(rename = "ps")]
    pub position_side: PerpSide,
}

impl From<BinanceFutOrderUpdate> for AccountEventKind {
    fn from(update: BinanceFutOrderUpdate) -> Self {
        AccountEventKind::Trade({
            let order = update.order;
            Trade {
                id: TradeId(order.trade_id.to_string()),
                order_id: OrderId(order.order_id.to_string()),
                instrument: Instrument {
                    base: order.symbol.clone(),
                    quote: order.symbol,
                    kind: InstrumentKind::Perpetual,
                },
                side: order.side,
                price: order.last_filled_price,
                quantity: order.order_last_filled_quantity,
                fees: SymbolFees {
                    symbol: order.commission_asset.unwrap_or_default(),
                    fees: order.commission.unwrap_or_default(),
                },
            }
        })
    }
}

// Test order update
#[cfg(test)]
mod test {
    use super::*;
    use barter_integration::model::instrument::symbol::Symbol;
    use chrono::Utc;

    #[test]
    fn test_order_update() {
        let order_update = BinanceFutOrderUpdate {
            event_type: BinanceFuturesEventType::OrderTradeUpdate,
            event_time: Utc::now(),
            transaction_time: Utc::now(),
            order: BinanceFutOrder {
                symbol: Symbol::new("BTCUSDT"),
                client_order_id: "TEST".to_string(),
                side: Side::Buy,
                order_type: BinFutOrderType::Limit,
                time_in_force: "GTC".to_string(),
                original_quantity: 0.001,
                original_price: 0.0,
                average_price: 0.0,
                stop_price: 7103.04,
                execution_type: BinFutExecutionType::New,
                order_status: BinanceFutOrderStatus::New,
                order_id: 8886774,
                order_last_filled_quantity: 0.0,
                order_filled_accumulated_quantity: 0.0,
                last_filled_price: 0.0,
                commission_asset: Some(Symbol::new("USDT")),
                commission: Some(0.0),
                order_trade_time: Utc::now(),
                trade_id: 0,
                bids_notional: 0.0,
                ask_notional: 9.91,
                is_maker_side: false,
                is_reduce_only: false,
                is_close_all: None,
                activation_price: None,
                callback_rate: None,
                is_price_protection: None,
                realized_profit: None,
                stp_mode: "EXPIRE_TAKER".to_string(),
                price_match_mode: "OPPONENT".to_string(),
                tif_gtd_order_auto_cancel_time: 0,
                position_side: PerpSide::Long,
            },
        };

        let account_event: AccountEventKind = order_update.into();
        match account_event {
            AccountEventKind::Trade(trade) => {
                assert_eq!(trade.id.0, "0");
                assert_eq!(trade.order_id.0, "8886774");
                assert_eq!(trade.instrument.base, "BTCUSDT".into());
                assert_eq!(trade.instrument.quote, "BTCUSDT".into());
                assert_eq!(trade.side, Side::Buy);
                assert_eq!(trade.price, 0.0);
                assert_eq!(trade.quantity, 0.0);
                assert_eq!(trade.fees.symbol, "USDT".into());
                assert_eq!(trade.fees.fees, 0.0);
            }
            _ => panic!("unexpected account event"),
        }
    }
}
