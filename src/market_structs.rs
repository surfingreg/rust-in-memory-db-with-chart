use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::*;

// use rust_decimal::Decimal;

#[derive(Deserialize, Serialize, Debug)]
pub struct Stat{
	pub dtg:DateTime<Utc>,
	pub dtg_last_tick:Option<DateTime<Utc>>,
	pub seq_last_tick:Option<u64>,
	pub price:Option<Decimal>,
	pub diff_ema:Option<Decimal>,
	pub diff_ema_roc:Option<Decimal>,
	pub spread:Option<Decimal>,
	pub ema1:Option<Decimal>,
	pub ema2:Option<Decimal>,
	pub min_sell:Option<Decimal>,
	pub max_buy:Option<Decimal>,

}

impl Stat{

	pub fn new()-> Self{
		Self {
			dtg:Utc::now(),
			dtg_last_tick:None,
			seq_last_tick:None,
			price:None,
			diff_ema:None,
			diff_ema_roc:None,
			spread:None,
			ema1:None,
			ema2:None,
			min_sell:None,
			max_buy:None,

		}
	}

	pub fn print(&self){
		println!("Stats:\tprice\t\tdiff_ema\t\tdiff_ema_roc\t\tspread\tema1\tema2\tmin_sell\tmax_buy\n\t{}\t{}\t{}\n\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", 
			self.dtg,
			self.dtg_last_tick.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(),
			self.seq_last_tick.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(),
			self.price.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(),
			self.diff_ema.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(), 
			self.diff_ema_roc.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(), 
			self.spread.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(),
			self.ema1.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(), 
			self.ema2.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(), 
			self.min_sell.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str(), 
			self.max_buy.map(|o| { format!("{}", o)}).unwrap_or("TBD".to_owned()).as_str()
		);		
	}
}


/*
#[derive(Deserialize, Serialize, Debug)]
pub struct TickerJson {
	// #[serde(rename = "type")]
	// pub ws_type: String,
	pub sequence:u64,
	// pub product_id:String,
	// pub price:String,
	pub price:Decimal,
	// pub open_24h:String,
	// pub volume_24h:String,
	// pub low_24h:String,
	// pub high_24h:String,
	// pub volume_30d:String,
	// pub best_bid:String,
	// pub best_ask:String,
	// pub side:String,
	#[serde(rename = "time")]
	// pub dtg:String,
	pub dtg:DateTime<Utc>,
	// pub trade_id:u64,
	// pub last_size:String,

}*/

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