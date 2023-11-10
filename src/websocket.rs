//! lib_websocket.rs

use std::error::Error;
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use tungstenite::{connect, Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use url::Url;
use crate::coinbase::Coinbase;


pub fn ws_connect() -> Result<(), Box<dyn Error>> {

    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    let url = std::env::var("COINBASE_URL").unwrap_or_else(|_| "wss://ws-feed.pro.coinbase.com".to_string());
    tracing::debug!("[websocket_go] url: {}", &url);
    let (mut socket, _) = connect(Url::parse(&url)?)?;
    ws_process(socket);
    Ok(())
}



fn ws_process(mut ws: WebSocket<MaybeTlsStream<TcpStream>>) {
    // let (mut ws, response) = tungstenite::connect(Url::parse(&url).unwrap()).unwrap();
    // tracing::info!("[websocket_go] lib_websocket connected, response: {:?}", response);

    // subscribe to coinbase.rs socket for heartbeat and tickers
    let _ = ws.send(Message::Text(generate_websocket_subscribe_json().to_string()));

    // parse incoming messages
    loop {
        let msg_result = ws.read();

        // Ok(Text("{\"type\":\"ticker\",\"sequence\":68161040101,\"product_id\":\"BTC-USD\",\"price\":\"36557.84\",\"open_24h\":\"35593.39\",\"volume_24h\":\"29347.72624298\",\"low_24h\":\"35555.16\",\"high_24h\":\"37999\",\"volume_30d\":\"413614.02343353\",\"best_bid\":\"36554.94\",\"best_bid_size\":\"0.02024396\",\"best_ask\":\"36557.84\",\"best_ask_size\":\"0.00875776\",\"side\":\"buy\",\"time\":\"2023-11-09T21:17:51.262478Z\",\"trade_id\":576007711,\"last_size\":\"0.00173305\"}"))

        match msg_result {
            Ok(Message::Text(t)) => {

                let json:Coinbase = serde_json::from_str(&t).unwrap();

                match json {
                    Coinbase::Subscriptions(s)=>{
                        tracing::debug!("[Coinbase::Subscriptions] {:?}", &s);
                    },
                    Coinbase::Ticker(t)=>{
                        tracing::debug!("[Coinbase::Ticker] {:?}", &t);
                    },
                    Coinbase::Heartbeat=>{
                        tracing::debug!("[ws][text] {:?}", &t);
                        tracing::debug!("[Coinbase::Heartbeat]");
                        panic!();
                    },
                    _ =>{
                        tracing::debug!("[ws][text] {:?}", &t);
                        tracing::error!("[Coinbase::something else] {:?}", &t);
                        panic!();
                    }
                }
            },
            _ => {}

            //
            //
            // Ok(msg) => {
            // 	match msg {
            // 		tungstenite::Message::Text(t) => {
            // 			let json_val:serde_json::Value = serde_json::from_str(&t).unwrap();
            // 			tracing::debug!("[ws] json_val: {:?}", &json_val);
            // 			let ws_type = json_val["type"].as_str();
            //
            // 			// Type
            // 			match ws_type {
            //
            // 				// TODO: make these types an enum
            //
            // 				Some("heartbeat") => {
            // 					tracing::debug!("[heartbeat]");
            // 				},
            //
            // 				Some("ticker") => {
            // 					// json to ticker
            // 					let ticker_opt:Option<TickerJson> = serde_json::from_value(json_val).expect("[ticker_actor] json conversion to Ticker 2 didn't work");
            //
            // 					if let Some(ticker) = ticker_opt {
            // 						self.process_ticker(ticker);
            // 					}
            // 				},
            //
            // 				Some("l2update") => {
            //
            // 					// parse json
            // 					let l2_update_opt: Option<UpdateL2> = serde_json::from_value(json_val).expect("[L2 Update] json conversion didn't work");
            //
            // 					// to database
            // 					if let Some(obj) = l2_update_opt {
            //
            // 						// tracing::debug!("[ws] {:?}", &obj);
            //
            // 						self.process_book_update(obj.changes);
            //
            // 					}
            // 				},
            // 				Some("snapshot") => {
            //
            // 					// tracing::debug!("[ws] snapshot: {:?}", json_val);
            //
            // 					let snapshot_opt:Option<Snapshot> = serde_json::from_value(json_val).expect("[ws:snapshot] json conversion didn't work");
            // 					// tracing::debug!("[ws] snapshot: {:?}", snapshot_opt);
            //
            // 					if snapshot_opt.is_some() {
            //
            // 						let snap:Snapshot = snapshot_opt.unwrap();
            //
            // 						for buy in &snap.bids {
            //
            // 							let _ = &self.book_buy.insert(buy.price.clone(), buy.size.clone());
            //
            // 						}
            //
            // 						for sell in &snap.asks {
            //
            // 							let _ = &self.book_sell.insert(sell.price.clone(), sell.size.clone());
            //
            // 						}
            // 					}
            // 				},
            // 				_ => {
            // 					tracing::debug!("[ws] unknown type: {:?}", json_val);
            // 				},
            // 			}
            // 		},
            // 		_ => {
            // 			tracing::info!("[main] unknown socket message or something not text")
            // 		}
            // 	}
            // },
            // Err(e) => {
            // 	match e {
            // 		tungstenite::error::Error::ConnectionClosed => {
            // 			// https://docs.rs/tungstenite/0.11.1/tungstenite/error/enum.Error.html#variant.ConnectionClosed
            // 			// TODO: stop the loop; attempt to reopen socket
            // 			tracing::info!("[parse_incoming_socket_blocking] socket: Error::ConnectionClosed");
            // 			break;
            // 		},
            // 		_ => {
            // 			tracing::info!("[parse_incoming_socket_blocking] socket read failed: {}", &e);
            // 			break;
            // 		}
            // 	}
            // },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Subscribe {
    #[serde(rename = "type")]
    pub typ : String,
    pub product_ids : Vec<String>,
    pub channels : Vec<String>
}

fn generate_websocket_subscribe_json() -> serde_json::Value {
    let cb_sub = Subscribe{
        typ:"subscribe".to_owned(),
        product_ids:vec!["BTC-USD".to_owned()],
        // channels:vec!["ticker".to_owned(), "level2".to_owned(), "user".to_owned()]
        // channels:vec!["ticker".to_owned(), "level2".to_owned()]
        channels:vec!["ticker".to_owned()]
    };
    let j : serde_json::Value = serde_json::to_value(&cb_sub).expect("[json_ws_subscribe] json serialize failed");
    j.to_owned()

}