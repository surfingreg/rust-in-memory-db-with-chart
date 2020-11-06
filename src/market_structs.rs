use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::*;
use crate::ticker::Ticker;




#[derive(Deserialize, Serialize, Debug, Clone)]
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

/*	pub fn print(&self){
		println!("Stats:\tprice\t\tdiff_ema\t\tdiff_ema_roc\t\tspread\tema1\tema2\tmin_sell\tmax_buy\n\t{}\t\t{}\t{}\n\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", 
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
	}*/

/*	pub fn format(&self)-> String{
		format!("Stats:\tprice\t\tdiff_ema\t\tdiff_ema_roc\t\tspread\tema1\tema2\tmin_sell\tmax_buy\n\t{}\t\t{}\t{}\n\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
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
		)
	}*/

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

#[derive(Deserialize, Serialize, Debug)]
pub struct SnapshotChange{
	pub price: Decimal,
	pub size: Decimal,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Snapshot {

	// price, size
	pub bids:Vec<SnapshotChange>,
	pub asks:Vec<SnapshotChange>
}

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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Trade{
	pub(crate) dtg: DateTime<Utc>,
	// id_buy: Uuid,
	// id_sell: Option<Uuid>,
	pub(crate) ticker_buy:Ticker,
	pub(crate) ticker_sell:Option<Ticker>,
	pub(crate) net_ticker:Decimal,

	// Market Theoretical
	// cost to buy/sell based on current book
	pub price_mkt_for_buy:Option<Decimal>,
	pub price_mkt_for_sell:Option<Decimal>,

	// amount even available to buy or sell
	pub size_mkt_for_buy:Option<Decimal>,
	pub size_mkt_for_sell:Option<Decimal>,

	pub price_net_actual:Decimal,

}

impl Trade{

	pub fn new(dtg: DateTime<Utc>, ticker_buy:Ticker, mkt_price_for_buy:Option<Decimal>, mkt_size_for_buy:Option<Decimal>) -> Self{
		Self{
			dtg: dtg,
			// id_buy: Uuid::new_v4(),
			// id_sell: None,
			ticker_buy:ticker_buy,
			ticker_sell:None,
			net_ticker:Decimal::from_u8(0).unwrap(),
			price_net_actual:Decimal::from_u8(0).unwrap(),
			price_mkt_for_buy: mkt_price_for_buy,
			price_mkt_for_sell:None,
			size_mkt_for_buy: mkt_size_for_buy,
			size_mkt_for_sell:None,
		}
	}

	pub fn new_with_sell(buy_trade:Trade, ticker_sell:Ticker, mkt_price_for_sell:Option<Decimal>, mkt_size_for_sell:Option<Decimal>) -> Self{
		Self{
			dtg: buy_trade.dtg.clone(),
			// id_buy: buy_trade.id_buy.clone(),
			ticker_buy:buy_trade.ticker_buy.clone(),
			// id_sell: Some(Uuid::new_v4()),
			ticker_sell:Some(ticker_sell.clone()),
			net_ticker:ticker_sell.price - &buy_trade.ticker_buy.price,
			price_net_actual:Decimal::from_u8(0).unwrap(),
			price_mkt_for_buy:buy_trade.price_mkt_for_buy,
			price_mkt_for_sell:mkt_price_for_sell,
			size_mkt_for_buy:buy_trade.size_mkt_for_buy,
			size_mkt_for_sell:mkt_size_for_sell,
		}
	}
}