use super::BinanceFuturesEventType;
use barter_integration::model::{instrument::symbol::Symbol, PerpSide};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::{
    balance::{Balance, SymbolBalance},
    AccountEventKind, Position,
};

/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) AccountUpdate messages.
///
/// ### Raw Payload Examples
/// See docs: <https://binance-docs.github.io/apidocs/futures/en/#event-balance-and-position-update>
/// ```json
/// {
///     "e": "ACCOUNT_UPDATE",                // Event Type
///     "E": 1564745798939,                   // Event Time
///     "T": 1564745798938 ,                  // Transaction
///     "a":                                  // Update Data
///       {
///         "m":"ORDER",                      // Event reason type
///         "B":[                             // Balances
///           {
///             "a":"USDT",                   // Asset
///             "wb":"122624.12345678",       // Wallet Balance
///             "cw":"100.12345678",          // Cross Wallet Balance
///             "bc":"50.12345678"            // Balance Change except PnL and Commission
///           },
///           {
///             "a":"BUSD",
///             "wb":"1.00000000",
///             "cw":"0.00000000",
///             "bc":"-49.12345678"
///           }
///         ],
///         "P":[
///           {
///             "s":"BTCUSDT",            // Symbol
///             "pa":"0",                 // Position Amount
///             "ep":"0.00000",           // Entry Price
///             "bep":"0",                // breakeven price
///             "cr":"200",               // (Pre-fee) Accumulated Realized
///             "up":"0",                 // Unrealized PnL
///             "mt":"isolated",          // Margin Type
///             "iw":"0.00000000",        // Isolated Wallet (if isolated position)
///             "ps":"BOTH"               // Position Side
///           }，
///           {
///             "s":"BTCUSDT",
///             "pa":"20",
///             "ep":"6563.66500",
///             "bep":"6563.6",
///             "cr":"0",
///             "up":"2850.21200",
///             "mt":"isolated",
///             "iw":"13200.70726908",
///             "ps":"LONG"
///           },
///           {
///             "s":"BTCUSDT",
///             "pa":"-10",
///             "ep":"6563.86000",
///             "bep":"6563.6",
///             "cr":"-45.04000000",
///             "up":"-1423.15600",
///             "mt":"isolated",
///             "iw":"6570.42511771",
///             "ps":"SHORT"
///           }
///         ]
///       }
///   }
/// ```

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceAccountUpdate {
    #[serde(alias = "e")]
    pub event_type: BinanceFuturesEventType,
    #[serde(
        alias = "E",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub event_time: DateTime<Utc>,
    #[serde(
        alias = "T",
        deserialize_with = "barter_integration::de::de_u64_epoch_ms_as_datetime_utc"
    )]
    pub transaction_time: DateTime<Utc>,
    #[serde(alias = "a")]
    pub update_data: BinanceAccountUpdateData,
}

/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) BinanceAccountUpdateData.
///
/// ### Raw Payload Examples
/// ```json
///         "m":"ORDER",                      // Event reason type
///         "B":[]                            // Balances
///         "P":[]
/// ```

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceAccountUpdateData {
    #[serde(alias = "m")]
    pub reason: String,
    #[serde(alias = "B")]
    pub balance_updates: Vec<BinanceBalanceUpdate>,
    #[serde(alias = "P")]
    pub position_updates: Vec<BinancePositionUpdate>,
}

/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) BinanceBalanceUpdate.
/// ### Raw Payload Examples
/// ```json
///          {
///             "a":"USDT",                   // Asset
///             "wb":"122624.12345678",       // Wallet Balance
///             "cw":"100.12345678",          // Cross Wallet Balance
///             "bc":"50.12345678"            // Balance Change except PnL and Commission
///      }
/// ```

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceBalanceUpdate {
    #[serde(alias = "a")]
    pub asset: Symbol,
    #[serde(alias = "wb", deserialize_with = "barter_integration::de::de_str")]
    pub wallet_balance: f64,
    #[serde(alias = "cw", deserialize_with = "barter_integration::de::de_str")]
    pub cross_wallet_balance: f64,
    #[serde(alias = "bc", deserialize_with = "barter_integration::de::de_str")]
    pub balance_change: f64,
}

/// [`BinanceFuturesUsd`](super::BinanceFuturesUsd) BinancePositionUpdate.
/// ### Raw Payload Examples
/// ```json
///           {
///             "s":"BTCUSDT",            // Symbol
///             "pa":"0",                 // Position Amount
///             "ep":"0.00000",           // Entry Price
///             "bep":"0",                // breakeven price
///             "cr":"200",               // (Pre-fee) Accumulated Realized
///             "up":"0",                 // Unrealized PnL
///             "mt":"isolated",          // Margin Type
///             "iw":"0.00000000",        // Isolated Wallet (if isolated position)
///             "ps":"BOTH"               // Position Side
///           }
/// ```

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinancePositionUpdate {
    #[serde(alias = "s")]
    pub symbol: Symbol,
    #[serde(alias = "pa", deserialize_with = "barter_integration::de::de_str")]
    pub position_amount: f64,
    #[serde(alias = "ep", deserialize_with = "barter_integration::de::de_str")]
    pub entry_price: f64,
    #[serde(alias = "bep", deserialize_with = "barter_integration::de::de_str")]
    pub breakeven_price: f64,
    #[serde(alias = "cr", deserialize_with = "barter_integration::de::de_str")]
    pub accumulated_realized: f64,
    #[serde(alias = "up", deserialize_with = "barter_integration::de::de_str")]
    pub unrealized_pnl: f64,
    #[serde(alias = "mt")]
    pub margin_type: String,
    #[serde(alias = "iw", deserialize_with = "barter_integration::de::de_str")]
    pub isolated_wallet: f64,
    #[serde(alias = "ps")]
    pub position_side: PerpSide,
}

impl From<BinanceBalanceUpdate> for SymbolBalance {
    fn from(balance_update: BinanceBalanceUpdate) -> Self {
        Self {
            symbol: balance_update.asset,
            balance: Balance {
                // TODO not totally clear if these are total or available
                total: balance_update.wallet_balance + balance_update.cross_wallet_balance,
                available: balance_update.wallet_balance + balance_update.cross_wallet_balance,
            },
        }
    }
}

impl From<BinancePositionUpdate> for Position {
    fn from(position_update: BinancePositionUpdate) -> Self {
        Self {
            // symbol: position_update.symbol,
            // position_amount: position_update.position_amount,
            // position_side: position_update.position_side,
            // unrealized_pnl: position_update.unrealized_pnl,
            // entry_price: position_update.entry_price,
            // breakeven_price: position_update.breakeven_price,
        }
    }
}

impl From<BinanceAccountUpdate> for (AccountEventKind, AccountEventKind) {
    fn from(update: BinanceAccountUpdate) -> Self {
        (
            AccountEventKind::Balances(
                update
                    .update_data
                    .balance_updates
                    .into_iter()
                    .map(SymbolBalance::from)
                    .collect(),
            ),
            AccountEventKind::Positions(
                update
                    .update_data
                    .position_updates
                    .into_iter()
                    .map(Position::from)
                    .collect(),
            ),
        )
    }
}

// impl From<(ExchangeId, Instrument, BinanceAccountUpdate)> for MarketIter<AccountUpdate> {
//     fn from(
//         (exchange_id, instrument, account_update): (ExchangeId, Instrument, BinanceAccountUpdate),
//     ) -> Self {
//         Self(vec![Ok(MarketEvent {
//             exchange_time: account_update.event_time,
//             received_time: Utc::now(),
//             exchange: Exchange::from(exchange_id),
//             instrument,
//             kind: AccountUpdate {
//                 time: account_update.event_time,
//                 balance_updates: account_update
//                     .update_data
//                     .balance_updates
//                     .into_iter()
//                     .map(BalanceUpdate::from)
//                     .collect(),
//                 position_updates: account_update
//                     .update_data
//                     .position_updates
//                     .into_iter()
//                     .map(PositionUpdate::from)
//                     .collect(),
//             },
//         })])
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    mod de {
        use super::*;
        use barter_integration::de::datetime_utc_from_epoch_duration;
        use std::time::Duration;

        #[test]
        fn test_binance_account_update() {
            let raw = r#"{
                "e": "ACCOUNT_UPDATE",
                "E": 1564745798939,
                "T": 1564745798938,
                "a": {
                    "m":"ORDER",
                    "B":[
                        {
                            "a":"USDT",
                            "wb":"122624.12345678",
                            "cw":"100.12345678",
                            "bc":"50.12345678"
                        },
                        {
                            "a":"BUSD",
                            "wb":"1.00000000",
                            "cw":"0.00000000",
                            "bc":"-49.12345678"
                        }
                    ],
                    "P":[
                        {
                            "s":"BTCUSDT",
                            "pa":"0",
                            "ep":"0.00000",
                            "bep":"0",
                            "cr":"200",
                            "up":"0",
                            "mt":"isolated",
                            "iw":"0.00000000",
                            "ps":"BOTH"
                        },
                        {
                            "s":"BTCUSDT",
                            "pa":"20",
                            "ep":"6563.66500",
                            "bep":"6563.6",
                            "cr":"0",
                            "up":"2850.21200",
                            "mt":"isolated",
                            "iw":"13200.70726908",
                            "ps":"LONG"
                        },
                        {
                            "s":"BTCUSDT",
                            "pa":"-10",
                            "ep":"6563.86000",
                            "bep":"6563.6",
                            "cr":"-45.04000000",
                            "up":"-1423.15600",
                            "mt":"isolated",
                            "iw":"6570.42511771",
                            "ps":"SHORT"
                        }
                    ]
                }
            }"#;

            let expected = BinanceAccountUpdate {
                event_type: BinanceFuturesEventType::AccountUpdate,
                event_time: datetime_utc_from_epoch_duration(Duration::from_millis(1564745798939)),
                transaction_time: datetime_utc_from_epoch_duration(Duration::from_millis(
                    1564745798938,
                )),
                update_data: BinanceAccountUpdateData {
                    reason: "ORDER".to_string(),
                    balance_updates: vec![
                        BinanceBalanceUpdate {
                            asset: Symbol::from("USDT"),
                            wallet_balance: 122624.12345678,
                            cross_wallet_balance: 100.12345678,
                            balance_change: 50.12345678,
                        },
                        BinanceBalanceUpdate {
                            asset: Symbol::from("BUSD"),
                            wallet_balance: 1.0,
                            cross_wallet_balance: 0.0,
                            balance_change: -49.12345678,
                        },
                    ],
                    position_updates: vec![
                        BinancePositionUpdate {
                            symbol: Symbol::from("BTCUSDT"),
                            position_amount: 0.0,
                            entry_price: 0.0,
                            breakeven_price: 0.0,
                            accumulated_realized: 200.0,
                            unrealized_pnl: 0.0,
                            margin_type: "isolated".to_string(),
                            isolated_wallet: 0.0,
                            position_side: PerpSide::Both,
                        },
                        BinancePositionUpdate {
                            symbol: Symbol::from("BTCUSDT"),
                            position_amount: 20.0,
                            entry_price: 6563.665,
                            breakeven_price: 6563.6,
                            accumulated_realized: 0.0,
                            unrealized_pnl: 2850.212,
                            margin_type: "isolated".to_string(),
                            isolated_wallet: 13200.70726908,
                            position_side: PerpSide::Long,
                        },
                        BinancePositionUpdate {
                            symbol: Symbol::from("BTCUSDT"),
                            position_amount: -10.0,
                            entry_price: 6563.86,
                            breakeven_price: 6563.6,
                            accumulated_realized: -45.04,
                            unrealized_pnl: -1423.156,
                            margin_type: "isolated".to_string(),
                            isolated_wallet: 6570.42511771,
                            position_side: PerpSide::Short,
                        },
                    ],
                },
            };
            let actual = serde_json::from_str::<BinanceAccountUpdate>(raw).unwrap();
            assert_eq!(actual, expected);
        }
    }
}
