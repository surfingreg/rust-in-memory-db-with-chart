//! common_lib...lib.rs
pub mod cb_ticker;
pub mod heartbeat;
pub mod init;
pub mod operator;

use chrono::{DateTime, Utc};
use datafusion::dataframe::DataFrame;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use tokio::sync::oneshot;
use crate::cb_ticker::Ticker;

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

// #[derive(Debug, Serialize, Deserialize)]
// pub struct CalcData {
//     pub calc_type: CalculationId,
//     pub x: DateTime<Utc>,
//     pub y: f64
// }







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
    RqstChartMulti {sender: oneshot::Sender<Vec<Chart2>> },
    RqstRaw {sender: oneshot::Sender<DataFrame> },

    // RequestChartJson{chart_type: ChartType, sender: oneshot::Sender<serde_json::Value> },
    // RequestChartRust{sender: oneshot::Sender<Chart> },

}

#[derive(Debug)]
pub enum ChartType{
    BasicAsJson,
    BasicAsRust,
    Test
}


#[derive(Debug, Deserialize, Display, Clone, EnumIter, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductId {
    #[serde(rename = "BTC-USD")]
    BtcUsd,
    #[serde(rename = "ETH-USD")]
    EthUsd,
    #[serde(rename = "ETH-BTC")]
    EthBtc,

}
impl ProductId {
    pub fn to_string_coinbase(&self) ->String{
        match self{
            ProductId::BtcUsd=>"BTC-USD".to_string(),
            ProductId::EthUsd=>"ETH-USD".to_string(),
            ProductId::EthBtc=>"ETH-BTC".to_string(),
        }
    }

}

#[derive(Debug, Serialize, Deserialize, Display, Clone, EnumIter, PartialEq)]
pub enum CalculationId {
    MovingAvg0004,
    MovingAvg0010,
    MovingAvg0100,
    MovingAvg1000,

}
impl CalculationId {
    pub fn to_string_coinbase(&self) ->String{
        match self{
            CalculationId::MovingAvg0004 => "mov_avg_0004".to_string(),
            CalculationId::MovingAvg0010 => "mov_avg_0010".to_string(),
            CalculationId::MovingAvg0100 => "mov_avg_0100".to_string(),
            CalculationId::MovingAvg1000 => "mov_avg_1000".to_string(),
        }
    }

    pub fn value(&self)-> usize {
        match self{
            CalculationId::MovingAvg0004 => 4,
            CalculationId::MovingAvg0010 => 10,
            CalculationId::MovingAvg0100 => 100,
            CalculationId::MovingAvg1000 => 1000,
        }
    }

}