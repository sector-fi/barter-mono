use barter_integration::model::{instrument::symbol::Symbol, PerpSide};
use chrono::{DateTime, Utc};

// Account update type
#[derive(Debug)]
pub struct AccountUpdate {
    pub time: DateTime<Utc>,
    pub balance_updates: Vec<BalanceUpdate>,
    pub position_updates: Vec<PositionUpdate>,
}

// Balance update type
#[derive(Debug)]
pub struct BalanceUpdate {
    pub asset: Symbol,
    pub wallet_balance: f64,
    pub cross_wallet_balance: f64,
    pub balance_change: f64,
}

// Position update type
#[derive(Debug)]
pub struct PositionUpdate {
    pub symbol: Symbol,
    pub position_amount: f64,
    pub position_side: PerpSide,
    pub unrealized_pnl: f64,
    pub entry_price: f64,
    pub breakeven_price: f64,
}
