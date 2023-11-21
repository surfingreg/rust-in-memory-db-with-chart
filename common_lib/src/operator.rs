//! operator.rs

use std::fmt::Debug;
use std::thread::JoinHandle;
use std::time::Duration;
use crossbeam_channel::{Sender, tick, unbounded};
use arrow_lib::arrow_db::{ArrowDbMsg, PriceEvent};
use crate::cb_ticker::Ticker;

const PING_MS:u64 = 10000;

#[derive(Debug)]
pub enum Msg {
    // Post(T),
    Post(Ticker),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
}

/// spawn a thread to listen for messages; return a way to send it crossbeam messages
pub fn run(_tx_db: Sender<ArrowDbMsg>) -> Sender<Msg> {

    let (tx,rx) = unbounded();
    let tx2 = tx.clone();

    std::thread::spawn(move ||{
        loop{
            match rx.recv(){
                Ok(message)=> process_message(&message, _tx_db.clone()),
                Err(e)=> tracing::debug!("[operator] error {:?}", &e),
            }
        }
    });
    let tx3 = tx2.clone();
    let _h = start_heartbeat(tx3);

    tx
}

fn process_message(message:&Msg, _tx_db: Sender<ArrowDbMsg>){
    match message{
        Msg::Ping => {
            tracing::debug!("[operator] PING");
        },
        Msg::Post(msg)=>{
            // tracing::debug!("[operator] LOG {:?}", &msg);

            _tx_db.send(ArrowDbMsg::Log(PriceEvent{
                dtg: msg.dtg.clone(),
                product_id: msg.product_id.to_string(),
                price: msg.price.clone(),
            })).unwrap();


        },
        _ => tracing::debug!("[operator] {:?} UNKNOWN ", &message)
    }
}

pub fn start_heartbeat(tx:Sender<Msg>) -> JoinHandle<()> {
    std::thread::spawn(move ||{
        let ticker = tick(Duration::from_millis(PING_MS));
        loop{
            tx.send(Msg::Ping).unwrap();
            ticker.recv().unwrap();
        }
    })
}