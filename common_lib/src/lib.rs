//! common_lib...lib.rs
use serde::{Deserialize, Serialize};

pub mod cb_ticker;
pub mod heartbeat;
pub mod init;
pub mod operator;

#[derive(Debug, Serialize, Deserialize)]
pub struct Chart{
    // chart_data: serde_json::Value
    pub columns: serde_json::Value,
    pub chart_data: serde_json::Value  // aka profit_total (daily)
}

#[derive(Debug, PartialEq)]
pub enum KitchenSinkError {
    ReqwestError,
    JsonError,
    Serde,
    ChannelError,
    Alpaca422,
    Alpaca429,
    Alpaca403,
    TransactionNotFound,
    BuyOrderExists,
    PositionExists,
    DeleteFailed,
    NoSharesFound,
    SqlInjectionRisk,
}