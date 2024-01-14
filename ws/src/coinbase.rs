//! coinbase.rs

use chrono::{DateTime, Utc};
use common_lib::cb_ticker::Ticker;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Coinbase {
    Subscriptions(Subscriptions),
    Heartbeat,
    Ticker(Ticker),
    // L2Update,
    // Snapshot,
    Error(Error),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel {
    name: String,
    product_ids: Vec<String>,
}

///    "{\"type\":\"subscriptions\",\"channels\":[{\"name\":\"ticker\",\"product_ids\":[\"BTC-USD\"]}]}"
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "Subscription")]
pub struct Subscriptions {

    channels: Vec<Channel>,
}

/// https://docs.cloud.coinbase.com/exchange/docs/websocket-errors
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Error {
    #[serde(rename = "time")]
    pub dtg: DateTime<Utc>,
    #[serde(rename="type")]
    coinbase_type:String,
    message:String,
}
