use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Ticker {
	// #[serde(rename = "type")]
	// pub ws_type: String,
	pub sequence:u64,
	// pub product_id:String,
	pub price:String,
	// pub open_24h:String,
	// pub volume_24h:String,
	// pub low_24h:String,
	// pub high_24h:String,
	// pub volume_30d:String,
	// pub best_bid:String,
	// pub best_ask:String,
	// pub side:String,
	pub time:String,
	pub trade_id:u64,
	// pub last_size:String,

}

#[derive(Deserialize, Serialize, Debug)]
pub struct Change {
	pub side: String,
	pub price: String,
	pub size: String,
}

// pub enum BookUpdate {
// 	Buy(Change),
// 	Sell(Change)
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateL2 {
	#[serde(rename = "type")]
	pub typ : String,
	// product_id : String,
	pub time:DateTime<Utc>,
	pub changes:Vec<Change>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Subscribe {
	#[serde(rename = "type")]
	pub typ : String,
	pub product_ids : Vec<String>,
	pub channels : Vec<String>
}