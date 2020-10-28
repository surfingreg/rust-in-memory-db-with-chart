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

	tx:crossbeam::channel::Sender<Msg>,

	stats:Vec<Stat>,

}

impl Market{

	fn new(tx:crossbeam::channel::Sender<Msg>) -> Self{
		Self{
			tickers : BTreeMap::new(),
			book_sell : BTreeMap::new(),
			book_buy : BTreeMap::new(),
			tx : tx,
			stats: vec![],
		}
	}

	pub fn start(){

		// Channel for websocket thread to send to database thread
		let (tx, rx) = crossbeam::channel::unbounded();

		// This
		let mut myself:Market = Market::new(tx);

		// Start Websocket
		let mut handles = vec![];
		handles.push (std::thread::spawn( move || {
			let _ws = myself.websocket_go();
		}));

		// Start Database
		handles.push(std::thread::spawn(move ||->(){
			let db_log_url = std::env::var("COIN_TRADE_LOG_DB_URL").expect("COIN_TRADE_LOG_DB_URL not found");
			if let Some(mut client) = Market::db_connect(&db_log_url){
				Market::db_thread(&mut client, rx);
			};
		}));

		for h in handles {
			h.join().unwrap();
		}
	}

	fn websocket_go(&mut self){

		println!("[websocket_go]");
		let url = std::env::var("COINBASE_URL").expect("COINBASE_URL must be set");
		println!("[websocket_go] url: {}", &url);
		let (mut ws, response) = tungstenite::connect(Url::parse(&url).unwrap()).unwrap();
		log::info!("[websocket_go] websocket connected, response: {:?}", response);

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
							// println!("[ws] json_val: {:?}", &json_val);

							let ws_type = json_val["type"].as_str();


							// Type
							match ws_type {

								Some("heartbeat") => {},

								Some("ticker") => {

									// json to ticker
									log::debug!("[ws] ticker"); /* {}", &ws_type.unwrap());*/
									// use as_str() to remove the quotation marks
									let ticker:Option<TickerJson> = serde_json::from_value(json_val).expect("[ticker_actor] json conversion to Ticker 2 didn't work"); // unwrap_or(None);

									if let Some(obj) = ticker {

										self.process_ticker(obj);

									}
								},
								Some("l2update") => {

									// parse json
									let l2_update_opt: Option<UpdateL2> = serde_json::from_value(json_val).expect("[L2 Update] json conversion didn't work");

									// to database
									if let Some(obj) = l2_update_opt {

										// log::debug!("[ws] {:?}", &obj);

										self.process_book_update(obj.changes);

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

	fn process_ticker(&mut self, mut t:TickerJson) {

		// calculate emas
		self.update_ticker_calcs(&mut t);

		// insert to tickers
		self.tickers.insert((&t).sequence, (&t).clone());

		// print all market stats
		&self.send_stat();

		// TODO: get trade recommendation firsts
		self.get_buy_bids_at_my_sell_price((&t).price, Decimal::from_f64(1.0).unwrap());
		self.get_sell_offers_at_my_buy_price((&t).price, Decimal::from_f64(1.0).unwrap());

	}



	/// I want to sell X.0 BTC at $Y.YY. What are the bids out there that would match my offer and quantity?
	/// Get the highest buy offers until my quantity is fill.
	fn get_buy_bids_at_my_sell_price(&self, my_price:Decimal, my_size:Decimal)/*-> Vec<Bid>*/{

		let mut matching_bids = vec![];
		let mut total_size = Decimal::from_u8(0).unwrap();

		// get all the bids to buy up to the quantity I want to sell
		// I want to sell 1.0 at $13000. So bids will probably be
		// -- 0.5 at 12999 and 0.5 at 12998
		// reverse() since we want the largest
		for (k_price,v_size) in self.book_buy.iter().rev() {
			matching_bids.push((k_price.clone(),v_size.clone()));
			total_size += v_size;
			if total_size >= my_size{
				break;
			}
		}

		println!("[get_buy_bids_at_my_sell_price] price: {}, size: {} => {:?}", my_price, my_size, matching_bids);

	}

	/// I want to buy X.0 BTC at $Y.YY. What are the lowest sell offers out there that would match my offer and quantity?
	/// Get the highest buy offers until my quantity is fill.
	fn get_sell_offers_at_my_buy_price(&self, my_price:Decimal, my_size:Decimal)/*-> Vec<Bid>*/{

		let mut matching_bids = vec![];
		let mut total_size = Decimal::from_u8(0).unwrap();

		// get all the bids to buy up to the quantity I want to sell
		// I want to sell 1.0 at $13000. So bids will probably be
		// -- 0.5 at 12999 and 0.5 at 12998
		// don't need to reverse for sell offers since we want the smallest
		for (k_price,v_size) in self.book_sell.iter() {

			matching_bids.push((k_price.clone(),v_size.clone()));
			total_size += v_size;
			if total_size >= my_size{
				break;
			}
		}

		println!("[get_sell_offers_at_my_buy_price] price: {}, size: {} => {:?}", my_price, my_size, matching_bids);

	}










	fn process_book_update(&mut self, changes:Vec<Change>){

		// TODO: what kind of sort does this imply? probably minor but print could be out of order, FYI
		for c in changes{

			// log::debug!("[process_book_update], {:?}", &c);

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
			&self.send_stat();
		}
	}

	fn build_stat_from_latest_ticker_and_book(&self) ->Option<Stat>{

		let mut stat = Stat::new(); 

		if !&self.book_buy.is_empty() && !&self.book_sell.is_empty()  {
	
			let max_buy = self.book_buy.last_key_value().unwrap().0;
			let min_sell = self.book_sell.first_key_value().unwrap().0;
			let spread = min_sell - max_buy;

			stat.spread =  Some(spread);
			stat.min_sell =  Some(*min_sell);
			stat.max_buy =  Some(*max_buy);
		}

		if !&self.tickers.is_empty(){
			let (_,ticker_v) = self.tickers.last_key_value().unwrap();
		
			stat.dtg_last_tick =  Some(ticker_v.dtg);
			stat.seq_last_tick =  Some(ticker_v.sequence);
			stat.price =  Some(ticker_v.price);
			stat.ema1 =  ticker_v.ema1;
			stat.ema2 =  ticker_v.ema2;
			stat.diff_ema =  ticker_v.diff_ema;
			stat.diff_ema_roc =  ticker_v.diff_ema_roc;

		}

		// println!("[latest_stat] Tickers: {}, Asks: {}, Bids: {}", &self.tickers.len(), &self.book_sell.len(), &self.book_buy.len());

		Some(stat)

	}

	/// Send the latest stat to the database
	/// Originally this printed the latest stat, potentially very inefficient, maybe send 1000 at a time?
	/// or send a ref to the stat map
	fn send_stat(&mut self) {
		if let Some(st) = self.build_stat_from_latest_ticker_and_book(){

			//let _ = self.tx.send(Msg::Statistic(st.clone()));

			self.stats.push(st);

			if self.stats.len() > 999 {
				let _ = self.tx.send(Msg::StatVector(self.stats.drain(..).collect()));
			};

		};
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

	/// Listen on a different thread for messages to persist to postgresql
	/// Each db network call takes 150-300ms on LAN/wifi
	pub fn db_thread(client: &mut postgres::Client,rcv:crossbeam::channel::Receiver<Msg>)-> JoinHandle<()>{
		loop {
			// this has the capacity to receive different messages from different transmitters, overkill right now
			crossbeam::channel::select!{
				recv(rcv) -> result => {
					if let Ok(msg) = &result {
						match msg {
/*							Msg::Statistic(stat) => {
								&stat.print();
							},
*/							Msg::StatVector(v) => {
								// for stat in v {
								// 	&stat.print();
								// };

								Market::stats_to_db(client, &v);

							},
							// _ => {},
						}
						// log::debug!("[db_thread] {}", &result_str);
					}
				}
			}
		}
	}

	/// Insert up to 1000 Stats in one network/database call
	fn stats_to_db(client:&mut postgres::Client , vec_stat:&Vec<Stat>){

		let start = std::time::Instant::now();

		// let mut sql = r#"insert
		// 	into t_stat (dtg, dtg_last_tick, price, diff_ema, diff_ema_roc, spread, ema1, ema2, min_sell, max_buy)
		// 	values ('2020-10-27 21:37:20.488852 UTC', '2020-10-27 21:37:20.488852 UTC', 0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0);"#;

		let insert = r#"insert into t_stat (dtg, dtg_last_tick, seq_last_tick, price, diff_ema, diff_ema_roc, spread, ema1, ema2, min_sell, max_buy) values"#;
		let mut sql = format!("{}", insert);

		for st in vec_stat {

			sql = format!("{} ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {} ), ", sql,
				st.dtg,
				st.dtg_last_tick.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.seq_last_tick.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.price.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.diff_ema.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.diff_ema_roc.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.spread.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.ema1.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.ema2.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.min_sell.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
				st.max_buy.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str()
			);
		}
		sql.pop();
		sql.pop();
		sql = format!("{};", &sql);

		println!("[stats_to_db] insert: {}, elapsed: {:?}", &vec_stat.len(), start.elapsed());

		let _ = client.simple_query(&sql);

	}

	pub fn db_connect(db_url:&str) -> Option<postgres::Client> {
		println!("[db_connect]");
		let client_result = postgres::Client::connect(&db_url, postgres::NoTls);
		match client_result {
			Err(e) 	=> {
				println!("[db_connect] postgresql connection failed: {}", &e);
				None
			},
			Ok(client) 	=> {
				println!("[db_connect] Connected to postgresql");
				Some(client)
			}
		}
	}

}
