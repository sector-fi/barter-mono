use serde::{Deserialize, Serialize};

pub mod account_update;
pub mod order_update;

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Serialize)]
pub enum BinanceFuturesEventType {
    AccountUpdate,
    OrderTradeUpdate,
    //     AccountConfigUpdate,
    //     MarginCall,
    //     ForceOrder,
    //     AccountPositionUpdate,
    //     LiquidationOrder,
    //     ContractPositionUpdate,
    //     ContractForceOrder,
    //     MarginCallForce,
    //     OrderBookTicker,
    //     Other,
}

impl<'de> Deserialize<'de> for BinanceFuturesEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "ACCOUNT_UPDATE" => Ok(Self::AccountUpdate),
            "ORDER_TRADE_UPDATE" => Ok(Self::OrderTradeUpdate),
            //             "ACCOUNT_CONFIG_UPDATE" => Ok(Self::AccountConfigUpdate),
            //             "MARGIN_CALL" => Ok(Self::MarginCall),
            //             "FORCE_ORDER" => Ok(Self::ForceOrder),
            //             "ACCOUNT_POSITION_UPDATE" => Ok(Self::AccountPositionUpdate),
            //             "LIQUIDATION_ORDER" => Ok(Self::LiquidationOrder),
            //             "CONTRACT_POSITION_UPDATE" => Ok(Self::ContractPositionUpdate),
            //             "CONTRACT_FORCE_ORDER" => Ok(Self::ContractForceOrder),
            //             "MARGIN_CALL_FORCE" => Ok(Self::MarginCallForce),
            //             "ORDER_BOOK_TICKER" => Ok(Self::OrderBookTicker),
            //             "OTHER" => Ok(Self::Other),
            _ => Err(serde::de::Error::custom(format!(
                "unknown BinanceFuturesEventType: {}",
                s
            ))),
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct BinanceFutAccountEvent {
    #[serde(alias = "e")]
    pub event_type: BinanceFuturesEventType,
}
