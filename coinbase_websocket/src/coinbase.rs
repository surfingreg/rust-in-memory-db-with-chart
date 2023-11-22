//! coinbase.rs

use serde::{Deserialize};
use common_lib::cb_ticker::Ticker;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Coinbase{
	Subscriptions(Subscriptions),
	Heartbeat,
	Ticker(Ticker),
	// L2Update,
	// Snapshot,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Channel{
	name:String,
	product_ids:Vec<String>
}

///    "{\"type\":\"subscriptions\",\"channels\":[{\"name\":\"ticker\",\"product_ids\":[\"BTC-USD\"]}]}"
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "Subscription")]
pub struct Subscriptions {
	channels:Vec<Channel>,
}

