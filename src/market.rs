// stats dtg, dtg_last_ticker to stats
// capture book on startup
// TODO: consider moving stats variables to object-level to make them available elsewhere (like in a message to a database)
// stats btreemap
// save stats to postgresql, but only once in a while
// prune tickers and book maps occasionally
// TODO: diff_ema_roc

use std::collections::BTreeMap;
use crate::market_structs::{*};
use crate::ticker::{Ticker as TickerJson} ;
use rust_decimal::prelude::*;
use std::thread::JoinHandle;
use tungstenite::{Message};
use url::Url;



pub struct Market{

	// btree is inherently sorted, key here is coinbase sequence so most recent is always fastest
	tickers:BTreeMap<u64, TickerJson>,

	// <price, size>
	book_sell:BTreeMap<Decimal, Decimal>,

	// <price, size>
	book_buy:BTreeMap<Decimal, Decimal>,



}

impl Market{

	fn new() -> Self{
		Self{
			tickers : BTreeMap::new(),
			book_sell : BTreeMap::new(),
			book_buy : BTreeMap::new(),
		}
	}

	pub fn start()-> JoinHandle<()>{
		std::thread::spawn( || {
			let mut myself:Market = Market::new();
			let _ws = myself.websocket_go();

		})
	}

	fn latest_stat(&self)->Option<Stat>{

		let mut stat = Stat::new(); 

		if !&self.book_buy.is_empty() && !&self.book_sell.is_empty()  {
	
			let max_buy = self.book_buy.last_key_value().unwrap().0;
			let min_sell = self.book_sell.first_key_value().unwrap().0;
			let spread = min_sell - max_buy;
			// last_key_value returns the max sorted by key

		
			// stat.dtg =  Utc::now(),
			// dtg_last_tick: Some(ticker_v.dtg),
			// seq_last_tick: Some(ticker_v.sequence),
			// price: Some(ticker_v.price),
			stat.spread =  Some(spread);
			// ema1 =  ticker_v.ema1,
			// ema2 =  ticker_v.ema2,
			// diff_ema =  ticker_v.diff_ema,
			// diff_ema_roc =  ticker_v.diff_ema_roc,
			stat.min_sell =  Some(*min_sell);
			stat.max_buy =  Some(*max_buy);
		}

		if !&self.tickers.is_empty(){
			let (_,ticker_v) = self.tickers.last_key_value().unwrap();
		
			// stat.dtg: Utc::now(),
			stat.dtg_last_tick =  Some(ticker_v.dtg);
			stat.seq_last_tick =  Some(ticker_v.sequence);
			stat.price =  Some(ticker_v.price);
			// stat.spread =  Some(spread),
			stat.ema1 =  ticker_v.ema1;
			stat.ema2 =  ticker_v.ema2;
			stat.diff_ema =  ticker_v.diff_ema;
			stat.diff_ema_roc =  ticker_v.diff_ema_roc;
			// stat.min_sell =  Some(*min_sell),
			// stat.max_buy =  Some(*max_buy),

		}

		println!("[latest_stat] Tickers: {}, Asks: {}, Bids: {}", &self.tickers.len(), &self.book_sell.len(), &self.book_buy.len());

		Some(stat)

	}

	fn print_stats(&self) {

		if let Some(st) = self.latest_stat(){
			st.print();
		};


	}

	fn websocket_go(&mut self){

		let url = std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");
		let (mut ws, _) = tungstenite::connect(Url::parse(&url).unwrap()).unwrap();

		log::info!("[main] websocket connected");

		// subscribe to coinbase socket for heartbeat and tickers
		let _ = ws.write_message(Message::Text(Market::json_ws_subscribe().to_string()));

		// parse incoming messages
		loop {
			let msg_result = ws.read_message();
			match msg_result {
				Err(e) => {
					match e {
						tungstenite::error::Error::ConnectionClosed => {
							// https://docs.rs/tungstenite/0.11.1/tungstenite/error/enum.Error.html#variant.ConnectionClosed
							// TODO: stop the loop; attempt to reopen socket
							log::info!("[parse_incoming_socket_blocking] socket: Error::ConnectionClosed");
							break;
						},
						_ => {
							log::info!("[parse_incoming_socket_blocking] socket read failed: {}", &e);
							break;
						}
					}
				},
				Ok(msg) => {
					match msg {
						tungstenite::Message::Text(t) => {

							let json_val:serde_json::Value = serde_json::from_str(&t).unwrap();

							let ws_type = json_val["type"].as_str();

							// Type
							match ws_type {

								Some("heartbeat") => {

								},

								Some("ticker") => {

									// json to ticker
									// use as_str() to remove the quotation marks
									let ticker:Option<TickerJson> = serde_json::from_value(json_val).expect("[ticker_actor] json conversion to Ticker 2 didn't work"); // unwrap_or(None);

									if let Some(mut obj) = ticker {
										// log::debug!("[ws] {:?}", &obj);
										// db_write_ticker(&t2, &mut pg_client);

										// calculate emas
										self.update_ticker_calcs(&mut obj);

										// Did it change, with EMAs?
										log::debug!("[ws] {:?}", &obj);

										// insert to tickers
										self.tickers.insert((&obj).sequence, (&obj).clone());


										// print all market stats
										&self.print_stats();
									}
								},
								Some("l2update") => {
									// parse json
									let l2_update_opt: Option<UpdateL2> = serde_json::from_value(json_val).expect("[L2 Update] json conversion didn't work");

									// to database
									if let Some(obj) = l2_update_opt {
										// log::debug!("[ws] {:?}", &obj);


										// TODO: what kind of sort does this imply? probably minor but print could be out of order, FYI
										for c in obj.changes{
											
											let size = Decimal::from_str(&c.size).unwrap();
											let size_is_zero = size == Decimal::from_u8(0).unwrap();
											let price = Decimal::from_str(&c.price).unwrap();
											

											if &c.side == "buy" {
												if size_is_zero {
													// if the size is zero, remove it
													&self.book_buy.remove(&price);
												} else {
													// if the size is not zero, add/replace what's in the btreemap
													&self.book_buy.insert(price, size);

												}

											} else if &c.side == "sell" {
												if size_is_zero {
													// if the size is zero, remove it
													&self.book_sell.remove(&price);
												} else {
													// if the size is not zero, add/replace what's in the btreemap
													&self.book_sell.insert(price, size);
												}
											}
											
											// print stats on every entry
											&self.print_stats();

										}

									}

								},

								Some("snapshot") => {

									// log::debug!("[ws] snapshot: {:?}", json_val);


									let snapshot_opt:Option<Snapshot> = serde_json::from_value(json_val).expect("[ws:snapshot] json conversion didn't work");
									// log::debug!("[ws] snapshot: {:?}", snapshot_opt);

									if snapshot_opt.is_some() {

										let snap:Snapshot = snapshot_opt.unwrap();

										for buy in &snap.bids {

											&self.book_buy.insert(buy.price.clone(), buy.size.clone());

										}

										for sell in &snap.asks {

											&self.book_sell.insert(sell.price.clone(), sell.size.clone());

										}

									}



								},
								_ => {
									log::debug!("[ws] unknown type: {:?}", json_val);
								},
							}
						},
						_ => {
							log::info!("[main] unknown socket message or something not text")
						}
					}
				}
			}
		}
	}

	fn json_ws_subscribe() -> serde_json::Value {
		let cb_sub = Subscribe{
			typ:"subscribe".to_owned(),
			product_ids:vec!["BTC-USD".to_owned()],
			// channels:vec!["ticker".to_owned(), "level2".to_owned(), "user".to_owned()]
			channels:vec!["ticker".to_owned(), "level2".to_owned()]
			// channels:vec!["ticker".to_owned()]
		};
		let j : serde_json::Value = serde_json::to_value(&cb_sub).expect("[json_ws_subscribe] json serialize failed");
		j.to_owned()
	
	}

	// TODO: not so functional, causes an effect
	fn update_ticker_calcs(&self, t: &mut TickerJson) {

		t.ema1 = self.compute_moving_average(5, t.price);
		t.ema2 = self.compute_moving_average(20, t.price);
		if t.ema1.is_some() && t.ema2.is_some() {
			t.diff_ema = Some(t.ema1.unwrap() - t.ema2.unwrap());
			t.diff_price_ema1 = Some(t.price - t.ema1.unwrap());
			t.diff_price_ema2 = Some(t.price - t.ema2.unwrap());
		}

		// log::debug("[update_ticker_calcs] {:?}", &t);

	}


	///
	/// get_ema20()
	///
	/// Calculate the 20 "day" exponential moving average
	/// n is the number of periods to include
	/// alpha (and 1 minus alpha) are the weights to give the current price and the previous average
	/// Sort the db descending, take the most recent n entries
	/// Do this before inserting the current price
	fn compute_moving_average(&self, n_days: usize, curr_price: Decimal) -> Option<Decimal> {

		// Exponential Moving Average variables
		// let n = 20;
		let smoothing_factor = Decimal::from_f64(2.0).unwrap();
		let alpha = smoothing_factor / Decimal::from_f64(1.0 + (n_days as f64)).unwrap();

		// sorted descending
		// Gets an iterator over the values of the map, in order by key.
		// https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.values
		let v: Vec<&TickerJson> = self.tickers.values().rev().collect();
		// println!("[compute_moving_average] \n{:?}", &v);

		// get a slice of the hashmap (for 5 day ema, len() could be 3, but only want 2, at index 1, 2 with 0 being current)
		let len_max = std::cmp::min(v.len(), n_days);
		match len_max {
			len_max if len_max > 1 => {

				let ema20_sum: Decimal = v[0..len_max].iter().fold(Decimal::from_u8(0).unwrap(), |sum, i| sum + i.price);

				let avg_of_n = ema20_sum / Decimal::from_usize(len_max).unwrap();

				// EMA = price (1-alpha) + average_previous_n (alpha)
				// TODO: confirm this shouldn't be the other way around, giving higher weight to the average than the current
				let ema_decimal = curr_price * (Decimal::from_usize(1).unwrap() - alpha) + avg_of_n * (alpha);

				Some(ema_decimal)
			},
			len_max if len_max == 1 => {
				None
			},
			_ => {
				// not possible
				None
			}
		}
	}
}
