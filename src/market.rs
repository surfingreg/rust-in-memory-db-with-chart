

use std::collections::BTreeMap;
use crate::market_structs::{Ticker, UpdateL2, Subscribe};
use rust_decimal::prelude::*;
use std::thread::JoinHandle;
use tungstenite::{Message};
use url::Url;



pub struct Market{

	// btree is inherently sorted, key here is coinbase sequence so most recent is always fastest
	tickers:BTreeMap<u64, Ticker>,

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

									// parse json
									// use as_str() to remove the quotation marks
									let ticker:Option<Ticker> = serde_json::from_value(json_val).expect("[ticker_actor] json conversion to Ticker 2 didn't work"); // unwrap_or(None);

									if let Some(obj) = ticker {
										log::debug!("[ws] {:?}", &obj);
										// db_write_ticker(&t2, &mut pg_client);
									}
								},
								Some("l2update") => {
									// parse json
									let l2_update_opt: Option<UpdateL2> = serde_json::from_value(json_val).expect("[L2 Update] json conversion didn't work");

									// to database
									if let Some(obj) = l2_update_opt {
										log::debug!("[ws] {:?}", &obj);

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
										}
										if !&self.book_buy.is_empty() && !&self.book_sell.is_empty() {

											println!("min sell: {:?}, max buy: {:?}", &self.book_sell.first_key_value(), &self.book_buy.last_key_value());
											println!("spread: {}", self.book_sell.first_key_value().unwrap().0 - self.book_buy.last_key_value().unwrap().0 );
										}
									}

								},

								Some("snapshot") => {
									log::debug!("[ws] snapshot: {:?}", json_val);
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
			channels:vec!["ticker".to_owned(), "level2".to_owned(), "user".to_owned()]
		};
		let j : serde_json::Value = serde_json::to_value(&cb_sub).expect("[json_ws_subscribe] json serialize failed");
		j.to_owned()
	}
}
