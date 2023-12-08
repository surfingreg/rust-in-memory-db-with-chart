//! common_lib...lib.rs
use datafusion::dataframe::DataFrame;
use serde::{Deserialize, Serialize};
use crate::cb_ticker::{ProductId, Ticker};
use tokio::sync::oneshot;

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


//
#[derive(Debug)]
pub enum Msg {
    // Post(T),
    Post(Ticker),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
    GetChartForOne {
        key: ProductId,
        sender: oneshot::Sender<VisualResultSet>,
    },
    GetChartForAll {
        key: ProductId,
        sender: oneshot::Sender<VisualResultSet>,
    },
    // TODO: pass a chart type with a single message instead of many possible messages
    RequestChart { chart_type: ChartType, sender: oneshot::Sender<Chart>},

}

#[derive(Debug)]
pub struct VisualResultSet {
    pub data: Option<DataFrame>,
    pub error: Option<String>,
}



#[derive(Debug)]
pub enum ChartType{
    Basic,
    Test
}