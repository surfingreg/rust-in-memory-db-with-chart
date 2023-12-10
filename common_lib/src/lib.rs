//! common_lib...lib.rs

use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::cb_ticker::{Ticker};
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

#[derive(thiserror::Error)]
#[derive(Debug, PartialEq, Display)]
pub enum KitchenSinkError {
    DbError,
    JsonError,
    Serde,
    ChannelError,
    SendError,
    RecvError,
    NoMessageMatch,
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
    // GetChartForOne {
    //     key: ProductId,
    //     sender: oneshot::Sender<VisualResultSet>,
    // },
    // GetChartForAll {
    //     key: ProductId,
    //     sender: oneshot::Sender<VisualResultSet>,
    // },
    // TODO: pass a chart type with a single message instead of many possible messages
    RequestChart{chart_type: ChartType, sender: oneshot::Sender<serde_json::Value> },
    // RequestChartJson{chart_type: ChartType, sender: oneshot::Sender<Chart> },

}

#[derive(Debug)]
pub enum ChartType{
    BasicAsJson,
    Test
}