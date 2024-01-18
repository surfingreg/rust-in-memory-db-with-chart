//! ws_inbound

#[allow(dead_code)]

use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::net::TcpStream;
use strum::IntoEnumIterator;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};
use common_lib::{DbMsg};
use common_lib::cb_ticker::{Datasource, SymbolCoinbase};
use chrono::{DateTime, Utc};
use common_lib::cb_ticker::TickerCoinbase;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbasePacket {
    Subscriptions(Subscriptions),
    Heartbeat,
    Ticker(TickerCoinbase),
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

/// Todo: make websocket post-processing asynchronous
pub fn parse(mut ws: WebSocket<MaybeTlsStream<TcpStream>>, tx_db: Sender<DbMsg>) {
    // subscribe to coinbase.rs socket for heartbeat and tickers
    let _ = ws.send(Message::Text(subscribe().to_string()));

    // parse incoming
    loop {
        let msg_result = ws.read();

        // Ok(Text("{\"type\":\"ticker\",\"sequence\":68161040101,\"product_id\":\"BTC-USD\",\"price\":\"36557.84\",\"open_24h\":\"35593.39\",\"volume_24h\":\"29347.72624298\",\"low_24h\":\"35555.16\",\"high_24h\":\"37999\",\"volume_30d\":\"413614.02343353\",\"best_bid\":\"36554.94\",\"best_bid_size\":\"0.02024396\",\"best_ask\":\"36557.84\",\"best_ask_size\":\"0.00875776\",\"side\":\"buy\",\"time\":\"2023-11-09T21:17:51.262478Z\",\"trade_id\":576007711,\"last_size\":\"0.00173305\"}"))
        match msg_result {
            Ok(Message::Text(t)) => {

                // TODO: coinbase can send something weird and crash the whole websocket with this unwrap

                match serde_json::from_str(&t) {

                    Ok(json)=>{
                        match json {
                            CoinbasePacket::Subscriptions(s) => tracing::debug!("[Coinbase::Subscriptions] {:?}", & s),
                            CoinbasePacket::Ticker(t) => {
                                if let Err(e) = tx_db.send(DbMsg::Insert(Datasource::Coinbase, t.to_common())) {
                                    tracing::error!("[ws_process] send error: {:?}", & e);
                                }
                            }
                            CoinbasePacket::Heartbeat => {
                                tracing::debug!("[ws_client][text] {:?}", & t);
                                tracing::debug!("[Coinbase::Heartbeat]");
                                panic!();
                            },
                            // 				Some("l2update") => {
                            // 					// parse json
                            // 					let l2_update_opt: Option<UpdateL2> = serde_json::from_value(json_val).expect("[L2 Update] json conversion didn't work");
                            //
                            // 					// to database
                            // 					if let Some(obj) = l2_update_opt {
                            // 						// tracing::debug!("[ws_client] {:?}", &obj);
                            // 						self.process_book_update(obj.changes);
                            // 					}
                            // 				},
                            // 				Some("snapshot") => {
                            // 					// tracing::debug!("[ws_client] snapshot: {:?}", json_val);
                            // 					let snapshot_opt:Option<Snapshot> = serde_json::from_value(json_val).expect("[ws_client:snapshot] json conversion didn't work");
                            // 					// tracing::debug!("[ws_client] snapshot: {:?}", snapshot_opt);
                            // 					if snapshot_opt.is_some() {
                            // 						let snap:Snapshot = snapshot_opt.unwrap();
                            // 						for buy in &snap.bids {
                            // 							let _ = &self.book_buy.insert(buy.price.clone(), buy.size.clone());
                            // 						}
                            // 						for sell in &snap.asks {
                            // 							let _ = &self.book_sell.insert(sell.price.clone(), sell.size.clone());
                            // 						}
                            // 					}
                            // 				},
                            CoinbasePacket::Error(error) => {
                                tracing::error!("[ws_process] coinbase error: {:?}", &error);
                            }
                        }
                    },
                    Err(e)=> {
                        tracing::error!("[ws_process] serde error: {:?}, \nmessage: {}", &e, &t);
                    }
                }
            }
            Err(e) => {

                tracing::error!("[ws_process] error: {:?}", &e);

                // https://docs.rs/tungstenite/0.21.0/tungstenite/error/enum.Error.html
                match e {
                    tungstenite::error::Error::AlreadyClosed | tungstenite::error::Error::ConnectionClosed => {
                        return;
                    },
                    _ => {}
                }
            }
            _ => {
                tracing::error!("[ws_process] non-text websocket data");
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Subscribe {
    #[serde(rename = "type")]
    pub typ: String,
    pub product_ids: Vec<String>,
    pub channels: Vec<String>,
}

/// here's where we subscribe to all the possible products (BTC-USD, ETH-USC, ETH-BTC
fn subscribe() -> serde_json::Value {

    let prod_ids = SymbolCoinbase::iter().map(|x|{x.to_string_coinbase()}).collect();

    let cb_sub = Subscribe {
        typ: "subscribe".to_owned(),
        // product_ids: vec!["BTC-USD".to_owned()],
        // product_ids: vec![ProductId::BtcUsd.to_string_coinbase()],
        product_ids: prod_ids,
        // channels:vec!["ticker".to_owned(), "level2".to_owned(), "user".to_owned()]
        // channels:vec!["ticker".to_owned(), "level2".to_owned()]
        channels: vec!["ticker".to_owned()],
    };
    let j: serde_json::Value = serde_json::to_value(cb_sub).expect("[json_ws_subscribe] json serialize failed");
    j.to_owned()
}
