//! operator.rs

use std::fmt::Debug;
use crossbeam_channel::{Sender, unbounded};
use datafusion::dataframe::DataFrame;
use tokio::sync::oneshot;
use crate::cb_ticker::{ProductId, Ticker};
use crate::heartbeat::start_heartbeat;

#[derive(Debug)]
pub enum Msg {
    // Post(T),
    Post(Ticker),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
    GetChartForOne {key:ProductId, sender: oneshot::Sender<VisualResultSet>},
    GetChartForAll {key:ProductId, sender: oneshot::Sender<VisualResultSet>},

}

#[derive(Debug)]
pub struct VisualResultSet{
    pub data: Option<DataFrame>,
    pub error: Option<String>,
}

/// spawn a thread to listen for messages; return a way to send it crossbeam messages
pub fn run(tx_db: Sender<Msg>) -> Sender<Msg> {
    let (tx,rx) = unbounded();
    let tx2 = tx.clone();
    std::thread::spawn(move ||{
        loop{
            match rx.recv(){
                Ok(message)=> process_message(message, tx_db.clone()),
                Err(e)=> tracing::debug!("[operator] error {:?}", &e),
            }
        }
    });
    let tx3 = tx2.clone();
    let _h = start_heartbeat(tx3);
    tx
}

/// not sure this is necessary
fn process_message(message:Msg, tx_db: Sender<Msg>){
    match message{
        Msg::Ping => tracing::debug!("[operator] PING"),
        Msg::Post(msg)=> tx_db.send(Msg::Post(msg)).unwrap(),
        Msg::GetChartForOne {key, sender} => {
            tracing::debug!("[operator] GetChartForOne");
            match tx_db.send(Msg::GetChartForOne { key, sender}){
                Ok(_)=>tracing::debug!("[operator] GetChartForOne sent to tx_db"),
                Err(e)=>tracing::error!("[operator] GetChartForOne send error: {:?}", &e),
            }
        },
        Msg::GetChartForAll {key, sender} => {
            tracing::debug!("[operator] GetChartForOne");
            match tx_db.send(Msg::GetChartForAll { key, sender}){
                Ok(_)=>tracing::debug!("[operator] GetChartForOne sent to tx_db"),
                Err(e)=>tracing::error!("[operator] GetChartForOne send error: {:?}", &e),
            }
        },
        _ => tracing::debug!("[operator] {:?} UNKNOWN ", &message)
    }
}

