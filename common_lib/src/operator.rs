//! operator.rs

use std::fmt::Debug;
use std::thread::JoinHandle;
use std::time::Duration;
use crossbeam_channel::{Sender, tick, unbounded};
use crate::cb_ticker::Ticker;

#[derive(Debug)]
pub enum Msg {
    // Post(T),
    Log(Ticker),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
}

/// spawn a thread to listen for messages; return a way to send it crossbeam messages
pub fn run() -> Sender<Msg> {

    let (tx,rx) = unbounded();
    let tx2 = tx.clone();

    std::thread::spawn(move ||{
        let tx3 = tx2.clone();
        let _h = start_heartbeat(tx3);

        loop{
            match rx.recv(){
                Ok(message)=> process_message(&message),
                Err(e)=> tracing::debug!("[operator] error {:?}", &e),
            }
        }
    });
    tx
}

fn process_message(message:&Msg){
    match message{
        Msg::Ping => {
            tracing::debug!("[operator] PING");
        },
        Msg::Log(msg)=>{
            tracing::debug!("[operator] LOG {:?}", &msg);
            // tracing::debug!("[Coinbase::Ticker] {:?}", &t);
        },
        _ => tracing::debug!("[operator] {:?} UNKNOWN ", &message)
    }
}

pub fn start_heartbeat(tx:Sender<Msg>) -> JoinHandle<()> {
    std::thread::spawn(move ||{
        let ticker = tick(Duration::from_millis(2000));
        loop{
            tx.send(Msg::Ping).unwrap();
            ticker.recv().unwrap();
        }
    })
}