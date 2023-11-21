//! arrow_db.rs

use std::fmt::Debug;
use std::thread::JoinHandle;
use std::time::Duration;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use crossbeam_channel::{Sender, tick, unbounded};

const PING_MS:u64 = 10000;

#[derive(Debug)]
pub enum ArrowDbMsg{
    // Post(T),
    Log(PriceEvent),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
}

#[allow(dead_code)]
#[derive(Debug)]
// #[serde(rename_all = "snake_case")]
pub struct PriceEvent{
    pub dtg:DateTime<Utc>,
    pub product_id:String,
    pub price:BigDecimal,
}

/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub fn run() -> Sender<ArrowDbMsg> {
    let (tx, rx) = unbounded();
    let tx2 = tx.clone();
    std::thread::spawn( move ||{
        let _ = start_heartbeat(tx2);
        loop{
            match rx.recv(){
                Ok(message)=> process_message(&message),
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

fn process_message(message:&ArrowDbMsg){
    match message{
        ArrowDbMsg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        ArrowDbMsg::Log(msg)=>{
            tracing::debug!("[arrow_db] POST {:?}", &msg);
            // tracing::debug!("[Coinbase::Ticker] {:?}", &t);
        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}


pub fn start_heartbeat(tx: Sender<ArrowDbMsg>) -> JoinHandle<()> {
    std::thread::spawn(move ||{
        let ticker = tick(Duration::from_millis(PING_MS));
        loop{
            tx.send(ArrowDbMsg::Ping).unwrap();
            // tracing::debug!("[main] sending ping to arrow_db");
            ticker.recv().unwrap();
        }
    })
}