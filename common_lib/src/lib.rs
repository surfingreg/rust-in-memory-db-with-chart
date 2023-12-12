//! common_lib...lib.rs

use chrono::{DateTime, Utc};
use datafusion::dataframe::DataFrame;
use serde::{Deserialize, Serialize};
use crate::cb_ticker::{Ticker};
use tokio::sync::oneshot;

pub mod cb_ticker;
pub mod heartbeat;
pub mod init;
pub mod operator;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartAsJson {
    // chart_data: serde_json::Value
    pub columns: serde_json::Value,
    pub chart_data: serde_json::Value  // aka profit_total (daily)
}

// TODO: generalize
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData {
    pub key:String,
    pub val:Vec<f64>

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chart {
    pub columns: Vec<DateTime<Utc>>,
    pub chart_data: Vec<ChartData>  // aka profit_total (daily)
}




#[derive(Debug, Serialize, Deserialize)]
pub struct Chart2 {
    pub label: String,
    pub data: Vec<ChartData2>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData2 {
    pub x: DateTime<Utc>,
    pub y: f64
}







#[derive(thiserror::Error)]
#[derive(Debug, PartialEq)]
pub enum KitchenSinkError {
    #[error("KitchenSinkError")]
    DbError,
    #[error("KitchenSinkError")]
    JsonError,
    #[error("KitchenSinkError")]
    Serde,
    #[error("KitchenSinkError")]
    RecvError,
    #[error("KitchenSinkError")]
    SendError,
    #[error("KitchenSinkError")]
    NoMessageMatch,
}


//
#[derive(Debug)]
pub enum Msg {
    // Post(T),
    Save(Ticker),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
    RequestChartJson{chart_type: ChartType, sender: oneshot::Sender<serde_json::Value> },
    RequestChartRust{sender: oneshot::Sender<Chart> },
    RequestChart2{sender: oneshot::Sender<Vec<Chart2>> },

    GetRaw{sender: oneshot::Sender<DataFrame> },

}

#[derive(Debug)]
pub enum ChartType{
    BasicAsJson,
    BasicAsRust,
    Test
}